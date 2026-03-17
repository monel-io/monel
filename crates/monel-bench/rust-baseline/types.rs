use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: u64,
    pub token: String,
    pub expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub failed_attempts: u32,
}
