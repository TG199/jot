use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TagName(String);

impl TagName {
    pub fn parse(s: String) -> Result<TagName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 50;

        let is_valid = s
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_');

        if is_empty_or_whitespace {
            Err("Tag name cannot be empty".to_string())
        } else if is_too_long {
            Err("Tag name is too long (max 50 characters".to_string())
        } else if !is_valid {
            Err("Tag name can only contain letters, numbers, hyphens, and underscores".to_string())
        } else {
            Ok(Self(s.to_lowercase()))
        }
    }
}

impl AsRef<str> for TagName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TagName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub tag_id: Uuid,
    pub user_id: Uuid,
    pub name: TagName,
}

#[derive(Debug, Clone)]
pub struct NewTag {
    pub user_id: Uuid,
    pub name: TagName,
}

impl NewTag {
    pub fn parse(user_id: Uuid, name: String) -> Result<NewTag, String> {
        let name = TagName::parse(name)?;
        Ok(Self { user_id, name })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn valid_tag_name_is_accepted() {
        assert_ok!(TagName::parse("work".to_string()));
        assert_ok!(TagName::parse("my-tag".to_string()));
        assert_ok!(TagName::parse("tag_123".to_string()));
    }

    #[test]
    fn empty_tag_name_is_rejected() {
        assert_err!(TagName::parse("".to_string()));
        assert_err!(TagName::parse("   ".to_string()));
    }

    #[test]
    fn tag_name_too_long_is_rejected() {
        let long_name = "a".repeat(51);
        assert_err!(TagName::parse(long_name));
    }

    #[test]
    fn tag_name_with_special_characters_is_rejected() {
        assert_err!(TagName::parse("tag name".to_string()));
        assert_err!(TagName::parse("tag@name".to_string()));
        assert_err!(TagName::parse("tag!".to_string()));
        assert_err!(TagName::parse("tag#".to_string()));
    }
}
