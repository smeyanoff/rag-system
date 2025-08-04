use std::fmt::Error;

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Document {
    pub id: Uuid,
    pub version: usize,
    pub text: String,
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait DocumentRepo: Send + Sync {
    async fn save(&self, doc: &Document) -> Result<(), Error>;
    async fn delete(&self, doc_id: Uuid) -> Result<(), Error>;
    async fn update(&self, doc: &Document) -> Result<(), Error>;
    async fn read(&self, doc_id: Uuid) -> Result<Document, Error>;
}

impl Document {
    pub fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            version: 1,
            text: text,
        }
    }

    pub fn update(&mut self, new_text: String) {
        self.version += 1;
        self.text = new_text;
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub text: String,
}

impl Chunk {
    pub fn new(doc_id: Uuid, text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            doc_id: doc_id,
            text: text,
        }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait ChunkRepo: Send + Sync {
    async fn save(&self, chunk: &Chunk) -> Result<(), Error>;
    async fn delete(&self, chunk_id: Uuid) -> Result<(), Error>;
    async fn read(&self, chunk_id: Uuid) -> Result<Chunk, Error>;
    async fn read_by_doc(&self, doc_id: Uuid) -> Result<Vec<Chunk>, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_updating() {
        let text = "hello";
        let check_text = "non hello";
        let mut document = Document::new(text.to_string());

        document.update(check_text.to_string());
        assert_eq!(document.text, check_text.to_string());
        assert_eq!(document.version, 2);
    }
}
