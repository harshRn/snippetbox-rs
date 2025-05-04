use sqlx::{
    MySql, Pool,
    types::chrono::{DateTime, Utc},
};

use std::error::Error;

use bcrypt::hash;
struct User {
    id: i32,
    name: String,
    email: String,
    hashed_password: Vec<u8>,
    created: DateTime<Utc>,
}

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

    pub fn authenticate(&self, email: &str, password: &str) -> Result<i32, sqlx::Error> {
        Ok(1)
    }

    pub fn exists(&self, id: i32) -> Result<bool, sqlx::Error> {
        Ok(false)
    }
}
