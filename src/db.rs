use crate::models::AgeSubmission;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn init_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

pub async fn store_age(
    pool: &Pool<Postgres>,
    submission: &AgeSubmission,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO encrypted_ages (user_id, encrypted_age) VALUES ($1, $2)",
        submission.user_id,
        submission.encrypted_age,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_encrypted_ages(pool: &Pool<Postgres>) -> Result<Vec<String>, sqlx::Error> {
    let records = sqlx::query!("SELECT encrypted_age FROM encrypted_ages")
        .fetch_all(pool)
        .await?;

    Ok(records.into_iter().map(|r| r.encrypted_age).collect())
}

pub async fn get_total_users(pool: &Pool<Postgres>) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!("SELECT COUNT(*) as count FROM encrypted_ages")
        .fetch_one(pool)
        .await?;

    Ok(record.count.unwrap_or(0))
}
