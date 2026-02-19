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
