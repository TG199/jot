use validator::validate_email;

#[derive(Debug, Clone)]
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
mod tests {}
