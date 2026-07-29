use loro::{LoroDoc, LoroMap, LoroValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: String,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct AuthoringProject {
    doc: LoroDoc,
    meta: ProjectMeta,
}

fn value_to_string(v: &LoroValue) -> Option<String> {
    if let LoroValue::String(s) = v {
        Some(s.to_string())
    } else {
        None
    }
}

impl AuthoringProject {
    pub fn new(title: &str) -> Self {
        let doc = LoroDoc::new();
        let now = chrono::Utc::now();
        let meta = ProjectMeta {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            created_at: now,
            updated_at: now,
        };
        let root = doc.get_map("root");
        root.insert("title", title).unwrap();
        root.insert("description", "").unwrap();
        Self { doc, meta }
    }

    pub fn load(bytes: &[u8]) -> Result<Self, AuthoringError> {
        let doc = LoroDoc::new();
        doc.import(bytes)
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        let title = doc
            .get_map("root")
            .get("title")
            .and_then(|v| value_to_string(&v.get_deep_value()))
            .unwrap_or_default();
        let meta = ProjectMeta {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        Ok(Self { doc, meta })
    }

    pub fn export(&self) -> Result<Vec<u8>, AuthoringError> {
        self.doc
            .export(loro::ExportMode::Snapshot)
            .map_err(|e| AuthoringError::Serialization(e.to_string()))
    }

    pub fn doc(&self) -> &LoroDoc {
        &self.doc
    }

    pub fn meta(&self) -> &ProjectMeta {
        &self.meta
    }

    pub fn set_title(&self, title: &str) -> Result<(), AuthoringError> {
        self.doc
            .get_map("root")
            .insert("title", title)
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        Ok(())
    }

    pub fn get_title(&self) -> Option<String> {
        let root = self.doc.get_map("root");
        root.get("title")
            .and_then(|v| value_to_string(&v.get_deep_value()))
    }

    pub fn set_description(&self, desc: &str) -> Result<(), AuthoringError> {
        self.doc
            .get_map("root")
            .insert("description", desc)
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        Ok(())
    }

    pub fn get_description(&self) -> Option<String> {
        self.doc
            .get_map("root")
            .get("description")
            .and_then(|v| value_to_string(&v.get_deep_value()))
    }

    pub fn add_chapter(&self, _index: usize, title: &str) -> Result<(), AuthoringError> {
        let chapters = self.doc.get_list("chapters");
        let text_key = format!("chapter_text_{}", chapters.len());
        let chapter_text = self.doc.get_text(text_key.as_str());
        let chapter_map = LoroMap::new();
        chapter_map
            .insert("title", title)
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        chapter_map
            .insert("text_key", text_key.as_str())
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        let _ = chapter_text;
        chapters
            .insert_container(chapters.len(), chapter_map)
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        Ok(())
    }

    pub fn remove_chapter(&self, index: usize) -> Result<(), AuthoringError> {
        self.doc
            .get_list("chapters")
            .delete(index, 1)
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        Ok(())
    }

    pub fn set_chapter_text(&self, _index: usize, text: &str) -> Result<(), AuthoringError> {
        let text_doc = self.doc.get_text("chapter_text");
        text_doc
            .insert(0, text)
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        Ok(())
    }

    pub fn get_chapter_text(&self, _index: usize) -> Option<String> {
        let text_doc = self.doc.get_text("chapter_text");
        Some(text_doc.to_string())
    }

    pub fn chapters_len(&self) -> usize {
        self.doc.get_list("chapters").len()
    }

    pub fn add_tag(&self, tag: &str) -> Result<(), AuthoringError> {
        self.doc
            .get_list("tags")
            .push(tag)
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        Ok(())
    }

    pub fn remove_tag(&self, tag: &str) -> Result<(), AuthoringError> {
        let tags = self.doc.get_list("tags");
        let len = tags.len();
        for i in 0..len {
            if let Some(v) = tags.get(i) {
                if value_to_string(&v.get_deep_value()).as_deref() == Some(tag) {
                    tags.delete(i, 1)
                        .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn get_tags(&self) -> Vec<String> {
        let tags = self.doc.get_list("tags");
        let len = tags.len();
        let mut result = Vec::new();
        for i in 0..len {
            if let Some(v) = tags.get(i) {
                if let Some(s) = value_to_string(&v.get_deep_value()) {
                    result.push(s);
                }
            }
        }
        result
    }

    pub fn merge(&self, other: &AuthoringProject) -> Result<(), AuthoringError> {
        let bytes = other
            .export()
            .map_err(|e| AuthoringError::Crdt(e.to_string()))?;
        self.doc
            .import(&bytes)
            .map(|_| ())
            .map_err(|e| AuthoringError::Crdt(e.to_string()))
    }

    pub fn apply_updates(&self, updates: &[u8]) -> Result<(), AuthoringError> {
        self.doc
            .import(updates)
            .map(|_| ())
            .map_err(|e| AuthoringError::Crdt(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthoringError {
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("CRDT error: {0}")]
    Crdt(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_project() {
        let p = AuthoringProject::new("Test Novel");
        assert_eq!(p.get_title().as_deref(), Some("Test Novel"));
    }

    #[test]
    fn add_and_get_chapter_works() {
        let p = AuthoringProject::new("Test");
        p.add_chapter(0, "Chapter 1").unwrap();
        p.set_chapter_text(0, "Once upon a time...").unwrap();
        let text = p.get_chapter_text(0);
        assert!(text.as_deref().unwrap_or("").contains("Once upon a time"));
    }

    #[test]
    fn tags_roundtrip() {
        let p = AuthoringProject::new("Test");
        p.add_tag("fantasy").unwrap();
        p.add_tag("romance").unwrap();
        let tags = p.get_tags();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"fantasy".to_string()));
    }

    #[test]
    fn export_import_roundtrip() {
        let p1 = AuthoringProject::new("Export Test");
        p1.add_chapter(0, "Ch1").unwrap();
        p1.set_chapter_text(0, "Hello world").unwrap();

        let bytes = p1.export().unwrap();
        let p2 = AuthoringProject::load(&bytes).unwrap();
        assert!(p2.chapters_len() > 0);
        assert_eq!(p2.get_title().as_deref(), Some("Export Test"));
    }

    #[test]
    fn two_replicas_converge() {
        let a = AuthoringProject::new("Converge");
        let b = AuthoringProject::new("Converge");

        a.add_chapter(0, "Ch1").unwrap();
        b.add_chapter(0, "Ch1").unwrap();

        a.set_chapter_text(0, "Edit A").unwrap();
        b.set_chapter_text(0, "Edit B").unwrap();

        a.merge(&b).unwrap();
        b.merge(&a).unwrap();

        assert_eq!(a.get_chapter_text(0), b.get_chapter_text(0));
    }
}
