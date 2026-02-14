use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct NoteContent(String);

impl NoteContent {
    pub fn parse(s: String) -> Result<NoteContent, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        if is_empty_or_whitespace {
            Err("Content cannot be empty".to_string())
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for NoteContent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NoteContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
