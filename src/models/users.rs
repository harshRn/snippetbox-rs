use sqlx::{
    MySql, Pool,
    types::chrono::{DateTime, Utc},
};

use std::{error::Error, f64::consts::E};

use bcrypt::hash;

use crate::models::errors::ErrInvalidCredentials;

struct User {
    id: i32,
    name: String,
    email: String,
    hashed_password: Vec<u8>,
    created: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug)]
struct UserRecord {
    id: i32,
    hashed_password: Vec<u8>,
}

#[derive(sqlx::FromRow, Debug)]
struct Exists {
    flag: i32,
}

#[derive(Clone)]
pub struct UserModel {
    pool: Pool<MySql>,
}

impl UserModel {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<u64, Box<dyn Error + Send>> {
        // error can pop-up during hashing or insertion.
        let hashed_password = hash(password, 12);
        if let Err(e) = hashed_password {
            tracing::error!(
                "encountered problems during user creation : password hashing failed : {}",
                e
            );
            return Err(Box::new(e));
        }

        let query = r#"INSERT INTO users (name, email, hashed_password, created)
                VALUES(?, ?, ?, UTC_TIMESTAMP())"#;
        match sqlx::query(query)
            .bind(name)
            .bind(email)
            .bind(hashed_password.unwrap())
            .execute(&self.pool)
            .await
        {
            Ok(r) => Ok(r.last_insert_id()),
            Err(e) => {
                // if e == sqlx::error::
                // check for duplicate record creation attempt
                tracing::error!("user could not be created : {}", e.to_string());
                Err(Box::new(e))
            }
        }
    }

    pub async fn authenticate(
        &self,
        email: &str,
        password: &str,
    ) -> Result<i32, Box<dyn Error + Send>> {
        let query = "SELECT id, hashed_password FROM users WHERE email = ?";
        match sqlx::query_as::<_, UserRecord>(query)
            .bind(email)
            .fetch_one(&self.pool)
            .await
        {
            Ok(res) => {
                match bcrypt::verify(
                    password,
                    String::from_utf8(res.hashed_password).unwrap().as_str(),
                ) {
                    Ok(x) => {
                        if x {
                            tracing::info!("login successful for {} ", email);
                            return Ok(res.id);
                        } else {
                            tracing::info!(
                                "login attempt failed due to invalid credentials for {}",
                                email
                            );
                            Err(Box::new(ErrInvalidCredentials))
                        }
                    }
                    Err(e) => {
                        if let bcrypt::BcryptError::InvalidHash(inv_pwd_msg) = &e {
                            tracing::info!(
                                "could not hash password correctly for user : {}, error: {}",
                                email,
                                inv_pwd_msg.clone()
                            );
                        }
                        return Err(Box::new(e));
                    }
                }
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    pub async fn exists(&self, id: i32) -> Result<bool, sqlx::Error> {
        let query = "SELECT EXISTS(SELECT true FROM users WHERE id = ?) as flag";
        let query_res = sqlx::query_as::<_, Exists>(query)
            .bind(id)
            .fetch_one(&self.pool)
            .await;
        match query_res {
            Ok(res) => {
                if res.flag == 1 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(e),
        }
    }
}
