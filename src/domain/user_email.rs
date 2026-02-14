use validator::ValidateEmail;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserEmail(String);

impl UserEmail {
    pub fn parse(s: String) -> Result<UserEmail, String> {
        if s.trim().is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        if !s.validate_email() {
            return Err(format!("'{}' is not a valid email address", s));
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for UserEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::UserEmail;
    use claims::{assert_err, assert_ok};

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(UserEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "jotdomain.com".to_string();
        assert_err!(UserEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(UserEmail::parse(email));
    }

    #[test]
    fn valid_email_is_parsed_successfully() {
        let email = "jot@domain.com".to_string();
        assert_ok!(UserEmail::parse(email));
    }
}
