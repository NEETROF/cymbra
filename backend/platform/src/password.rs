//! argon2id password hashing + policy (task 2.10).

use crate::error::{AppError, Result};
use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

/// Hash a password with argon2id (random salt). Returns the PHC string to store.
pub fn hash(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2 hash failed: {e}")))
}

/// Verify `password` against a stored argon2id PHC hash.
pub fn verify(password: &str, phc: &str) -> bool {
    match PasswordHash::new(phc) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Enforce the configurable password policy (currently a minimum length).
pub fn check_policy(password: &str, min_length: usize) -> Result<()> {
    if password.chars().count() < min_length {
        return Err(AppError::InvalidArgument(format!(
            "password must be at least {min_length} characters"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify() {
        let h = hash("correct horse battery staple").unwrap();
        assert!(verify("correct horse battery staple", &h));
        assert!(!verify("wrong password", &h));
    }

    #[test]
    fn policy_rejects_short() {
        assert!(check_policy("short", 12).is_err());
        assert!(check_policy("a-long-enough-passphrase", 12).is_ok());
    }
}
