use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use secrecy::{ExposeSecret, SecretString};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserPassWord(SecretString);

impl UserPassWord {
    pub fn parse(s: String) -> Result<UserPassWord, String> {
        if s.len() < 8 {
            return Err("Password must be atleast 8 characters long".to_string());
        }

        if s.len() > 128 {
            return Err("Password must be at most 128 characters long".to_string());
        }

        if !s.chars().any(|c| c.is_ascii_digit()) {
            return Err("Password must contain at least one digit".to_string());
        }

        if !s.chars().any(|c| c.is_ascii_alphabetic()) {
            return Err("Password must contain at least one letter".to_string());
        }

        Ok(Self(SecretString::new(s.into())))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

pub fn compute_password_hash(password: &UserPassWord) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::default()
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &UserPassWord,
) -> Result<(), anyhow::Error> {
    let expected_password_hash = PasswordHash::new(expected_password_hash)
        .map_err(|e| anyhow::anyhow!("Failed to parse hash in PHC string format: {}", e))?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .map_err(|e| anyhow::anyhow!("Invalid password: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn password_too_short_is_rejected() {
        let password = "jshort".to_string();

        assert_err!(UserPassWord::parse(password));
    }

    #[test]
    fn password_too_long_is_rejected() {
        let password = "a".repeat(129) + "1";

        assert_err!(UserPassWord::parse(password));
    }
    #[test]
    fn password_without_digit_is_rejected() {
        let password = "nodigitshere".to_string();
        assert_err!(UserPassWord::parse(password));
    }

    #[test]
    fn password_without_letter_is_rejected() {
        let password = "12345678".to_string();
        assert_err!(UserPassWord::parse(password));
    }

    #[test]
    fn valid_password_is_parsed_successfully() {
        let password = "validPass123".to_string();
        assert_ok!(UserPassWord::parse(password));
    }

    #[test]
    fn password_hash_verification_works() {
        let password = UserPassWord::parse("testPass123".to_string()).unwrap();
        let hash = compute_password_hash(&password).unwrap();

        assert_ok!(verify_password_hash(&hash, &password));
    }

    #[test]
    fn wrong_password_fails_verification() {
        let password = UserPassWord::parse("testPass123".to_string()).unwrap();
        let wrong_password = UserPassWord::parse("wrongPass123".to_string()).unwrap();
        let hash = compute_password_hash(&password).unwrap();
        assert_err!(verify_password_hash(&hash, &wrong_password));
    }
}
