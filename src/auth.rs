use crate::models::{MagicLink, User};
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn find_or_create_user(pool: &SqlitePool, email: &str) -> Result<User> {
    let email = email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(anyhow!("invalid email"));
    }

    if let Some(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await?
    {
        return Ok(user);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id, email) VALUES (?, ?)")
        .bind(&id)
        .bind(&email)
        .execute(pool)
        .await?;

    Ok(User {
        id,
        email,
        created_at: Utc::now().to_rfc3339(),
    })
}

pub async fn create_magic_link(pool: &SqlitePool, user_id: &str) -> Result<String> {
    let raw_token = generate_token();
    let token_hash = hash_token(&raw_token);
    let expires = (Utc::now() + Duration::minutes(30)).to_rfc3339();

    sqlx::query(
        "INSERT INTO magic_links (token, user_id, expires_at) VALUES (?, ?, ?)",
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(&expires)
    .execute(pool)
    .await?;

    Ok(raw_token)
}

pub async fn verify_magic_link(pool: &SqlitePool, raw_token: &str) -> Result<User> {
    let token_hash = hash_token(raw_token);

    let link = sqlx::query_as::<_, MagicLink>(
        "SELECT * FROM magic_links WHERE token = ? AND used = 0",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow!("invalid or expired link"))?;

    let expires = chrono::DateTime::parse_from_rfc3339(&link.expires_at)
        .map_err(|_| anyhow!("bad expiry"))?;
    if expires < Utc::now() {
        return Err(anyhow!("link expired"));
    }

    sqlx::query("UPDATE magic_links SET used = 1 WHERE token = ?")
        .bind(&token_hash)
        .execute(pool)
        .await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&link.user_id)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

pub fn session_cookie_value(user_id: &str) -> String {
    user_id.to_string()
}
