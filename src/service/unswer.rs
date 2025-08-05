use std::fmt::Error;
use std::sync::Arc;

use uuid::Uuid;

use futures::future::join_all;
use tokio::sync::Mutex;

use crate::domain::{
    document::ChunkRepo,
    embedding::{QuestionEmbeddingRepo, VectorSearcher},
    question::QuestionRepo,
    unswer::{LLM, Unswer, UnswerRepo},
};

pub struct UnswerService {
    llm: Arc<dyn LLM>,
    unswer_repo: Arc<dyn UnswerRepo>,
    question_repo: Arc<dyn QuestionRepo>,
    question_embeding_repo: Arc<dyn QuestionEmbeddingRepo>,
    vector_searcher: Arc<dyn VectorSearcher>,
    chunk_repo: Arc<dyn ChunkRepo>,
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl UnswerService {
    pub fn new(
        llm: Arc<dyn LLM>,
        unswer_repo: Arc<dyn UnswerRepo>,
        question_repo: Arc<dyn QuestionRepo>,
        question_embeding_repo: Arc<dyn QuestionEmbeddingRepo>,
        vector_searcher: Arc<dyn VectorSearcher>,
        chunk_repo: Arc<dyn ChunkRepo>,
        semaphore: Arc<tokio::sync::Semaphore>,
    ) -> Self {
        Self {
            llm,
            unswer_repo,
            question_repo,
            question_embeding_repo,
            vector_searcher,
            chunk_repo,
            semaphore: semaphore,
        }
    }
}

impl UnswerService {
    pub async fn get_unswer(&self, question_id: Uuid, similar_k: usize) -> Result<String, Error> {
        // Клонируем зависимости
        let question_repo = self.question_repo.clone();
        let question_emb_repo = self.question_embeding_repo.clone();
        let vector_searcher = self.vector_searcher.clone();
        let chunk_repo = self.chunk_repo.clone();
        let semaphore = self.semaphore.clone();
        let llm = self.llm.clone();
        let unswer_repo = self.unswer_repo.clone();

        // Запрашиваем вопрос и эмбеддинг параллельно
        let question_handle = tokio::spawn(async move { question_repo.read(question_id).await });

        let embedding_handle =
            tokio::spawn(async move { question_emb_repo.read(question_id).await });

        let question = question_handle.await.unwrap()?;
        let question_embedding = embedding_handle.await.unwrap()?;

        // Ищем похожие чанки
        let k_nearest = vector_searcher
            .search_similar(&question_embedding.vec, similar_k)
            .await?;

        // Загружаем чанки параллельно
        let context = Arc::new(Mutex::new(Vec::<String>::new()));
        let mut chunk_handles = Vec::with_capacity(k_nearest.len());

        for chunk_id in k_nearest.clone() {
            let chunk_repo = chunk_repo.clone();
            let semaphore = semaphore.clone();
            let context = context.clone();

            chunk_handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let chunk = chunk_repo.read(chunk_id).await?;
                context.lock().await.push(chunk.text);
                Ok::<(), Error>(())
            }));
        }

        // Дожидаемся всех чанков
        join_all(chunk_handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let context = context.lock().await.clone();

        // Формируем ответ
        let unswer_text = llm.formulate_unswer(question.text, context).await?;

        // Сохраняем ответ
        let unswer = Unswer::new(unswer_text.clone(), k_nearest);
        unswer_repo.save(&unswer).await?;

        Ok(unswer_text)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::question::{MockQuestionRepo, Question};
    use crate::domain::embedding::{MockQuestionEmbeddingRepo, QuestionEmbending, MockVectorSearcher};
    use crate::domain::document::{MockChunkRepo, Chunk};
    use crate::domain::unswer::{MockLLM, MockUnswerRepo};


    #[tokio::test]
    async fn test_get_unswer_happy_path() {
        // Данные
        let question_id = Uuid::new_v4();
        let chunk_id = Uuid::new_v4();
        let doc_id = Uuid::new_v4();
        let response_text = "Rust is a safe and fast language.".to_string();

        // Моки
        let mut mock_question_repo = MockQuestionRepo::new();
        mock_question_repo
            .expect_read()
            .returning(move |_| Ok(Question::new("What is Rust?".into())));

        let mut mock_embedding_repo = MockQuestionEmbeddingRepo::new();
        mock_embedding_repo
            .expect_read()
            .returning(move |_| Ok(QuestionEmbending{id: Uuid::new_v4(), question_id: question_id.clone(), vec: vec![0.1, 0.2] }));

        let mut mock_vector_searcher = MockVectorSearcher::new();
        mock_vector_searcher
            .expect_search_similar()
            .returning(move |_, _| Ok(vec![chunk_id]));

        let mut mock_chunk_repo = MockChunkRepo::new();
        mock_chunk_repo
            .expect_read()
            .returning(move |_| Ok(Chunk::new(doc_id.clone(), "Rust is a programming language.".into())));

        let mut mock_llm = MockLLM::new();
        let resp_clone = response_text.clone();
        mock_llm
            .expect_formulate_unswer()
            .returning(move |_, _| Ok(resp_clone.clone()));

        let mut mock_unswer_repo = MockUnswerRepo::new();
        mock_unswer_repo
            .expect_save()
            .returning(|_| Ok(()));

        // Сервис (условный конструктор)
        let service = UnswerService {
            question_repo: Arc::new(mock_question_repo),
            question_embeding_repo: Arc::new(mock_embedding_repo),
            vector_searcher: Arc::new(mock_vector_searcher),
            chunk_repo: Arc::new(mock_chunk_repo),
            semaphore: Arc::new(tokio::sync::Semaphore::new(5)),
            llm: Arc::new(mock_llm),
            unswer_repo: Arc::new(mock_unswer_repo),
        };

        // Вызов
        let result = service.get_unswer(question_id, 1).await.unwrap();

        // Проверка
        assert_eq!(result, response_text);
    }

    
}
