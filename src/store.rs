use sqlx::{postgres::{PgPoolOptions, PgRow}, Pool, Postgres, Row};

use tracing::{error, info};
use crate::types::account::{Account, AccountId};

use crate::types::question::{Question, QuestionId, QuestionPayload};

#[derive(Debug, Clone)]
pub struct Store {
  pub pool: Pool<Postgres>,
}

impl Store {
  pub async fn new(db_url: &str) -> Self {
    let pool = PgPoolOptions::new()
      .max_connections(5)
      .connect(db_url)
      .await
      .expect("Could not establish new connection!!!");
    Store { pool }
  }

  pub async fn get_q(
    &self,
    limit: Option<i32>,
    offset: i32,
  ) -> Result<Vec<Question>, sqlx::Error> {
    let qs = sqlx::query("SELECT * FROM questions LIMIT $1 OFFSET $2")
      .bind(limit)
      .bind(offset)
      .map(|row: PgRow| {
        let id = row.get::<i32, _>("id");
        Question {
          id: QuestionId(id as u32),
          title: row.get("title"),
          content: row.get("content"),
          tags: row.get("tags"),
        }
      })
      .fetch_all(&self.pool)
      .await;

    match qs {
      Ok(q) => Ok(q),
      Err(e) => {
        error!("Failed to get q: {:?}", e);
        Err(e)
      }
    }
  }

  pub async fn add_q(&self, q: QuestionPayload) -> Result<Question, sqlx::Error> {
    sqlx::query(
      "INSERT INTO questions (title, content, tags)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, tags",
    )
      .bind(q.title)
      .bind(q.content)
      .bind(q.tags)
      .map(|row: PgRow| {
        let id = row.get::<i32, _>("id");
        Question {
          id: QuestionId(id as u32),
          title: row.get("title"),
          content: row.get("content"),
          tags: row.get("tags"),
        }
      })
      .fetch_one(&self.pool)
      .await
  }

  pub async fn del_q(&self, id: i32) -> Result<i32, sqlx::Error> {
    sqlx::query("DELETE FROM questions WHERE id = $1 RETURNING id")
      .bind(id)
      .map(|row: PgRow| row.get::<i32, _>("id"))
      .fetch_one(&self.pool)
      .await
  }

  pub async fn upd_q(&self, id: i32, q: QuestionPayload) -> Result<i32, sqlx::Error> {
    sqlx::query(
      "UPDATE questions SET title = $1, content = $2, tags = $3 WHERE id = $4 RETURNING id",
    )
      .bind(q.title)
      .bind(q.content)
      .bind(q.tags)
      .bind(id)
      .map(|row: PgRow| row.get("id"))
      .fetch_one(&self.pool)
      .await
  }

  pub(crate) async fn add_a(&self, qid: i32, content: String) -> Result<bool, sqlx::Error> {
    match sqlx::query(
      "INSERT INTO answers(content, corresponding_question) VALUES ($1, $2) RETURNING id",
    )
      .bind(content)
      .bind(qid)
      .map(|row: PgRow| row.get::<i32, _>("id"))
      .fetch_one(&self.pool)
      .await
    {
      Ok(id) => {
        info!("New answer [{id}] created");
        Ok(true)
      }
      Err(e) => Err(e),
    }
  }

  pub async fn add_account(&self, a: Account) -> Result<i32, sqlx::Error> {
    match sqlx::query(
      "INSERT INTO account(email, password) VALUES ($1, $2) RETURNING id",
    )
      .bind(a.email)
      .bind(a.password)
      .map(|row: PgRow| row.get::<i32, _>("id"))
      .fetch_one(&self.pool)
      .await
    {
      Ok(id) => {
        info!("New account with id=[{id}] is created");
        Ok(id)
      }
      Err(e) => Err(e),
    }
  }

  pub async fn find_account(&self, email: String) -> Result<Account, sqlx::Error> {
    sqlx::query("SELECT * FROM account WHERE email = $1")
      .bind(email)
      .map(|row: PgRow| Account {
        id: Some(AccountId(row.get("id"))),
        email: row.get("email"),
        password: row.get("password"),
      })
      .fetch_one(&self.pool)
      .await
  }
}
