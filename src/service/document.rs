use std::fmt::Error;
use std::sync::Arc;

use uuid::Uuid;

use crate::domain::document::{Chunk, ChunkRepo, Document, DocumentRepo};
use crate::domain::embedding::{ChunkEmbending, ChunkEmbendingRepo, TextVectorizer};

pub struct DocumentService {
    pub max_chunk_size: usize,
    document_repo: Arc<dyn DocumentRepo>,
    chunk_repo: Arc<dyn ChunkRepo>,
    embending_vectorizer: Arc<dyn TextVectorizer>,
    embending_repo: Arc<dyn ChunkEmbendingRepo>,
}

impl DocumentService {
    pub fn new(
        max_chunk_size: usize,
        document_repo: Arc<dyn DocumentRepo>,
        chunk_repo: Arc<dyn ChunkRepo>,
        embending_vectorizer: Arc<dyn TextVectorizer>,
        embending_repo: Arc<dyn ChunkEmbendingRepo>,
    ) -> Self {
        DocumentService {
            max_chunk_size: max_chunk_size,
            document_repo: document_repo,
            chunk_repo: chunk_repo,
            embending_vectorizer: embending_vectorizer,
            embending_repo: embending_repo,
        }
    }
}

impl DocumentService {
    fn prepare_document(&self, document: &Document) -> Vec<Chunk> {
        let text = document.text.as_str();
        let mut chunks = Vec::new();

        let mut current_chunk = String::new();

        for p_chunk in text.split("\n").map(|x| x.trim()) {
            for c in p_chunk.chars() {
                if current_chunk.len() + c.len_utf8() >= self.max_chunk_size {
                    chunks.push(Chunk::new(document.id, current_chunk.to_string()));
                    current_chunk = String::new();
                }
                current_chunk.push(c);
            }
        }

        // добавить последний чанк, если остался
        if !current_chunk.trim().is_empty() {
            chunks.push(Chunk::new(document.id, current_chunk.to_string()));
        }

        chunks
    }
}

impl DocumentService {
    pub fn process_new_document(&self, document: &str) -> Result<(), Error> {
        // prepare document
        let document = Document::new(document.to_string());
        self.document_repo.save(&document)?;
        // prepare chunks
        let chunks = self.prepare_document(&document);
        for chunk in chunks.iter() {
            self.chunk_repo.save(chunk)?;
        }
        // prepare embendings
        for chunk in chunks.iter() {
            let embending = ChunkEmbending::new(chunk, self.embending_vectorizer.as_ref())?;
            self.embending_repo.save(&embending)?;
        }
        Ok(())
    }

    pub fn update_document(&self, document_id: Uuid, new_document: &str) -> Result<(), Error> {
        // get the document
        let mut document = self.document_repo.read(document_id)?;
        for chunk in self.chunk_repo.read_by_doc(document_id)? {
            self.chunk_repo.delete(chunk.id)?;
            self.embending_repo.delete(chunk.id)?;
        }
        // update document
        document.update(new_document.to_string());
        // update chunks
        for chunk in self.prepare_document(&document).iter() {
            self.chunk_repo.save(chunk)?;
            let embedding = ChunkEmbending::new(chunk, self.embending_vectorizer.as_ref())?;
            self.embending_repo.save(&embedding)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_document() {
        let text = "
        Мороз и солнце; день чудесный!
        Еще ты дремлешь, друг прелестный —
        Пора, красавица, проснись:
        Открой сомкнуты негой взоры
        Навстречу северной Авроры,
        Звездою севера явись!

        Вечор, ты помнишь, вьюга злилась,
        На мутном небе мгла носилась;
        Луна, как бледное пятно,
        Сквозь тучи мрачные желтела,
        И ты печальная сидела —
        А нынче… погляди в окно:";

        let doc_repo = Arc::new(crate::domain::document::MockDocumentRepo::new());
        let chunk_repo = Arc::new(crate::domain::document::MockChunkRepo::new());
        let vectorizer = Arc::new(crate::domain::embedding::MockTextVectorizer::new());
        let emb_repo = Arc::new(crate::domain::embedding::MockChunkEmbendingRepo::new());

        let document = Document::new(text.to_string());
        let service = DocumentService::new(128, doc_repo, chunk_repo, vectorizer, emb_repo);
        let chunks = service.prepare_document(&document);

        assert!(!chunks.is_empty(), "Chunks should not be empty");

        for chunk in &chunks {
            assert_eq!(chunk.doc_id, document.id);
            assert!(!chunk.text.is_empty(), "Chunk text should not be empty");
            assert!(
                chunk.text.len() <= 128,
                "Chunk size should not exceed limit"
            );
        }
    }
}
