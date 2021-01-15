use crate::utils;
use deadpool_postgres::{Pool, PoolError};

pub const POLL_ID_LENGTH: usize = 8;
pub type PollID = String;

macro_rules! poll_duration {
    () => { "INTERVAL '1 day'" }
}

pub struct PollNum {
    pub title: String,
    pub minimum: f64,
    pub maximum: f64,
    pub integer: bool,
}

fn is_integer(n: f64) -> bool {
    (n as i64) as f64 == n
}

pub fn valid_poll_num(config: &PollNum) -> bool {
    if config.title.len() == 0 || config.title.len() > 128 { return false; }
    if config.minimum >= config.maximum { return false; }
    if config.integer {
        if config.minimum == -f64::INFINITY || is_integer(config.minimum) { return false; }
        if config.maximum == f64::INFINITY || is_integer(config.maximum) { return false; }
    }
    true
}

fn generate_poll_id_num() -> PollID {
    format!("n{}", utils::generate_random_base64url(POLL_ID_LENGTH - 1))
}

pub async fn create_poll_num(pool: Pool, config: PollNum) -> Result<PollID, PoolError> {
    let conn = pool.get().await?;
    let stmt = conn.prepare("
        INSERT INTO poll_numerical (poll_id, creation_time, title, minimum, maximum, only_integers)
        VALUES ($1, NOW(), $2, $3, $4, $5)
        ON CONFLICT (poll_id) DO NOTHING
    ").await?;

    let mut poll_id = generate_poll_id_num();
    while conn.execute(&stmt, &[&poll_id, &config.title, &config.minimum, &config.maximum, &config.integer]).await? == 0 {
        poll_id = generate_poll_id_num();
    }

    Ok(poll_id)
}

pub async fn get_poll_num(pool: Pool, poll_id: PollID) -> Result<Option<PollNum>, PoolError> {
    let conn = pool.get().await?;
    let stmt = conn.prepare(concat!("
        SELECT title, minimum, maximum, only_integers
        FROM poll_numerical
        WHERE poll_id = $1
        AND creation_time > NOW() - ", poll_duration!())
    ).await?;
    Ok(conn.query_opt(&stmt, &[&poll_id]).await?.map(|row| PollNum {
        title: row.get(0),
        minimum: row.get(1),
        maximum: row.get(2),
        integer: row.get(3),
    }))
}

pub struct ResponseNum(f64);

pub async fn respond_poll_num(pool: Pool, poll_id: PollID, res: ResponseNum) -> Result<(), PoolError> {
    let conn = pool.get().await?;
    let stmt = conn.prepare("
        INSERT INTO poll_numerical_response (poll_id, value)
        VALUES ($1, $2)
    ").await?;
    conn.execute(&stmt, &[&poll_id, &res.0]).await?;
    Ok(())
}
