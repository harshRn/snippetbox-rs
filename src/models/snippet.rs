use sqlx::{
    Executor, MySql, Pool, pool,
    types::chrono::{DateTime, Utc},
};

pub struct Snippet {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
}

pub struct SnippetModel {
    pool: Pool<MySql>,
}

impl SnippetModel {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        title: &str,
        content: &str,
        expires: i32,
    ) -> Result<u64, sqlx::Error> {
        let query = format!(
            r#"INSERT INTO snippets (title, content, created, expires)
                VALUES('{}', '{}', UTC_TIMESTAMP(), DATE_ADD(UTC_TIMESTAMP(), INTERVAL {} DAY))"#,
            title, content, expires
        );

        match self.pool.execute(query.as_str()).await {
            Ok(r) => Ok(r.last_insert_id()),
            Err(e) => {
                tracing::error!("record could not be inserted : {}", e);
                return Err(e);
            }
        }
    }

    // async fn get(&self, id: i32) -> Snippet {
    //     Snippet {
    //         id: (),
    //         title: (),
    //         content: (),
    //         created: (),
    //         expires: (),
    //     }
    // }

    // async fn latest(&self) -> Result<Vec<Snippet>, Err> {}
}
