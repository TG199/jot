use crate::domain::note_content::NoteContent;
use crate::domain::note_title::NoteTitle;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Note {
    pub note_id: Uuid,
    pub user_id: Uuid,
    pub title: NoteTitle,
    pub content: NoteContent,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewNote {
    pub user_id: Uuid,
    pub title: NoteTitle,
    pub content: NoteContent,
}

impl NewNote {
    pub fn parse(user_id: Uuid, title: String, content: String) -> Result<NewNote, String> {
        let title = NoteTitle::parse(title)?;
        let content = NoteContent::parse(content)?;

        Ok(Self {
            user_id,
            title,
            content,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UpdateNote {
    pub title: Option<NoteTitle>,
    pub content: Option<NoteContent>,
}

impl UpdateNote {
    pub fn parse(title: Option<String>, content: Option<String>) -> Result<UpdateNote, String> {
        let title = match title {
            Some(t) => Some(NoteTitle::parse(t)?),
            None => None,
        };

        let content = match content {
            Some(c) => Some(NoteContent::parse(c)?),
            None => None,
        };

        if title.is_none() && content.is_none() {
            return Err("At least one field (title or content) must be provided".to_string());
        }
        Ok(Self { title, content })
    }
}
