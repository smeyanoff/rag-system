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
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl DocumentService {
    pub fn new(
        max_chunk_size: usize,
        document_repo: Arc<dyn DocumentRepo>,
        chunk_repo: Arc<dyn ChunkRepo>,
        embending_vectorizer: Arc<dyn TextVectorizer>,
        embending_repo: Arc<dyn ChunkEmbendingRepo>,
        semaphore: Arc<tokio::sync::Semaphore>,
    ) -> Self {
        DocumentService {
            max_chunk_size: max_chunk_size,
            document_repo: document_repo,
            chunk_repo: chunk_repo,
            embending_vectorizer: embending_vectorizer,
            embending_repo: embending_repo,
            semaphore: semaphore,
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
    pub async fn process_new_document(&self, document: &str) -> Result<(), Error> {
        // 1. Сохраняем сам документ
        let document = Document::new(document.to_string());
        self.document_repo.save(&document).await?;

        // 2. Разбиваем на чанки
        let chunks = self.prepare_document(&document);

        // 3. Создаём семафор для ограничения параллелизма
        let mut handles = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let semaphore = self.semaphore.clone();
            let chunk_repo = self.chunk_repo.clone();
            let embending_repo = self.embending_repo.clone();
            let vectorizer = self.embending_vectorizer.clone();

            handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await.unwrap();

                // Сохраняем чанк
                chunk_repo.save(&chunk).await?;

                // Генерируем эмбеддинг
                let embending = ChunkEmbending::new(&chunk, vectorizer.as_ref()).await?;

                // Сохраняем эмбеддинг
                embending_repo.save(&embending).await?;

                Ok::<(), Error>(())
            }));
        }

        // 4. Ждём выполнения всех задач
        for handle in handles {
            handle.await.unwrap()?;
        }

        Ok(())
    }

    pub async fn update_document(
        &self,
        document_id: Uuid,
        new_document: &str,
    ) -> Result<(), Error> {
        // 1. Получаем документ и старые чанки
        let mut document = self.document_repo.read(document_id).await?;
        let doc_chunks = self.chunk_repo.read_by_doc(document_id).await?;

        // 2. Удаляем старые чанки и эмбеддинги параллельно
        let mut delete_handles = Vec::with_capacity(doc_chunks.len());
        for chunk in doc_chunks {
            let semaphore = self.semaphore.clone();
            let chunk_repo = self.chunk_repo.clone();
            let embedding_repo = self.embending_repo.clone();

            delete_handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await;
                chunk_repo.delete(chunk.id).await?;
                embedding_repo.delete(chunk.id).await?;
                Ok::<(), Error>(())
            }));
        }
        for h in delete_handles {
            h.await.unwrap()?;
        }

        // 3. Обновляем документ
        document.update(new_document.to_string());

        // 4. Готовим новые чанки
        let chunks = self.prepare_document(&document);

        // 5. Параллельно векторизуем + сохраняем
        let mut vectorize_handles = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let semaphore = self.semaphore.clone();
            let chunk_repo = self.chunk_repo.clone();
            let embedding_repo = self.embending_repo.clone();
            let vectorizer = self.embending_vectorizer.clone();

            vectorize_handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await;

                // сохраняем чанк
                chunk_repo.save(&chunk).await?;

                // векторизация (REST)
                let embedding = ChunkEmbending::new(&chunk, vectorizer.as_ref()).await?;

                // сохраняем эмбеддинг
                embedding_repo.save(&embedding).await?;
                Ok::<(), Error>(())
            }));
        }

        // ждем всех
        for h in vectorize_handles {
            h.await.unwrap()?;
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
        let semaphore = Arc::new(tokio::sync::Semaphore::new(5));

        let document = Document::new(text.to_string());
        let service =
            DocumentService::new(128, doc_repo, chunk_repo, vectorizer, emb_repo, semaphore);
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
