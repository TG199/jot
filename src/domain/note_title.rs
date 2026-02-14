use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct NoteTitle(String);

impl NoteTitle {
    pub fn parse(s: String) -> Result<NoteTitle, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 200;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contians_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace {
            Err("Title cannot be empty".to_string())
        } else if is_too_long {
            Err("Title is too long (max 200 characters)".to_string())
        } else if contians_forbidden_characters {
            Err("Title contains forbidden characters".to_string())
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for NoteTitle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NoteTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
