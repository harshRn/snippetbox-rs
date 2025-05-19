use sqlx::{
    MySql, Pool,
    types::chrono::{DateTime, Utc},
};

#[derive(sqlx::FromRow, Debug)]
pub struct Snippet {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SnippetModel {
    pool: Pool<MySql>,
}

impl SnippetModel {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        title: String,
        content: String,
        expires: i32,
    ) -> Result<u64, sqlx::Error> {
        let query = r#"INSERT INTO snippets (title, content, created, expires) 
                VALUES (?, ?, UTC_TIMESTAMP(), DATE_ADD(UTC_TIMESTAMP(), INTERVAL ? DAY))"#;

        match sqlx::query(query)
            .bind(title)
            .bind(content)
            .bind(expires)
            .execute(&self.pool)
            .await
        {
            Ok(r) => Ok(r.last_insert_id()),
            Err(e) => {
                tracing::error!("record could not be inserted : {}", e);
                return Err(e);
            }
        }
    }

    pub async fn get(&self, id: &u32) -> Result<Snippet, sqlx::Error> {
        let query = r#"SELECT id, title, content, created, expires FROM snippets
            WHERE expires > UTC_TIMESTAMP() AND id = ?"#;
        match sqlx::query_as::<_, Snippet>(query)
            .bind(id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(res) => Ok(res),
            Err(e) => {
                if let sqlx::error::Error::RowNotFound = e {
                    tracing::error!("record could not be found : {}", e.to_string());
                } else {
                    tracing::error!("query failed : {}", e.to_string());
                }
                Err(e)
            }
        }
    }

    pub async fn latest(&self) -> Result<Vec<Snippet>, sqlx::Error> {
        let query = r#"SELECT id, title, content, created, expires FROM snippets
            WHERE expires > UTC_TIMESTAMP() ORDER BY id DESC LIMIT 10"#;

        match sqlx::query_as::<_, Snippet>(query)
            .fetch_all(&self.pool)
            .await
        {
            Ok(r) => Ok(r),
            Err(e) => {
                tracing::error!("some problem : {}", e.to_string());
                Err(e)
            }
        }
        // defer rows.Close()
    }
}
