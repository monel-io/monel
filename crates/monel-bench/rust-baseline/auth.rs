use crate::types::{Credentials, Session, User};
use crate::errors::AuthError;
use std::time::{Duration, Instant};

// In real code these would be trait objects or async database calls.
// The AI agent has to read this entire file to understand what authenticate does,
// what side effects it has, and what errors it can return.

/// Authenticates a user with the given credentials.
///
/// Returns a session on success, or an auth error on failure.
pub fn authenticate(
    creds: &Credentials,
    db: &dyn AuthDb,
    crypto: &dyn CryptoVerifier,
) -> Result<Session, AuthError> {
    let user = db.find_user_by_username(&creds.username);

    let user = match user {
        None => {
            log::info!("auth_failed: user_not_found");
            return Err(AuthError::InvalidCreds);
        }
        Some(u) => u,
    };

    if user.failed_attempts >= 5 {
        log::info!("auth_blocked: account_locked, user_id={}", user.id);
        return Err(AuthError::Locked);
    }

    if !crypto.verify(&creds.password, &user.password_hash) {
        db.increment_failed_attempts(user.id);
        log::info!("auth_failed: invalid_password, user_id={}", user.id);
        return Err(AuthError::InvalidCreds);
    }

    db.reset_failed_attempts(user.id);

    if let Some(existing) = db.find_active_session(user.id) {
        if existing.expires_at <= Instant::now() {
            db.delete_session(&existing.token);
            log::info!("auth_failed: session_expired, user_id={}", user.id);
            return Err(AuthError::Expired);
        }
    }

    let session = Session {
        user_id: user.id,
        token: crypto.generate_token(),
        expires_at: Instant::now() + Duration::from_secs(86400),
    };

    db.insert_session(&session);
    log::info!("auth_success: user_id={}", user.id);
    Ok(session)
}

pub trait AuthDb {
    fn find_user_by_username(&self, username: &str) -> Option<User>;
    fn find_active_session(&self, user_id: u64) -> Option<Session>;
    fn insert_session(&self, session: &Session);
    fn delete_session(&self, token: &str);
    fn increment_failed_attempts(&self, user_id: u64);
    fn reset_failed_attempts(&self, user_id: u64);
}

pub trait CryptoVerifier {
    fn verify(&self, password: &str, hash: &str) -> bool;
    fn generate_token(&self) -> String;
}
