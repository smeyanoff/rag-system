use std::fmt::Error;
use std::sync::Arc;

use crate::domain::embedding::{QuestionEmbeddingRepo, QuestionEmbending, TextVectorizer};
use crate::domain::question::{Question, QuestionRepo};

pub struct QuestionService {
    question_repo: Arc<dyn QuestionRepo>,
    embedding_repo: Arc<dyn QuestionEmbeddingRepo>,
    vectorizer: Arc<dyn TextVectorizer>,
}

impl QuestionService {
    pub fn new(
        question_repo: Arc<dyn QuestionRepo>,
        embedding_repo: Arc<dyn QuestionEmbeddingRepo>,
        vectorizer: Arc<dyn TextVectorizer>,
    ) -> Self {
        Self {
            question_repo: question_repo,
            embedding_repo: embedding_repo,
            vectorizer: vectorizer,
        }
    }
}

impl QuestionService {
    pub async fn process_new_question(&self, text: &str) -> Result<(), Error> {
        let question = Arc::new(Question::new(text.to_string()));
        let question_clone = question.clone();
        let repo_clone = self.question_repo.clone();

        // Сохраняем вопрос
        let save_handle = tokio::spawn(async move {
            repo_clone.save(&question_clone).await?;
            Ok::<(), Error>(())
        });

        let vectorizer = self.vectorizer.clone();
        let embedding_repo = self.embedding_repo.clone();
        let question_clone2 = question.clone();

        // Создаем и сохраняем эмбеддинг
        let embed_handle = tokio::spawn(async move {
            let question_embedding =
                QuestionEmbending::new(&question_clone2, vectorizer.as_ref()).await?;
            embedding_repo.save(&question_embedding).await?;
            Ok::<(), Error>(())
        });

        // Ждём обе задачи (или можно не ждать, если "fire and forget")
        save_handle.await.unwrap()?;
        embed_handle.await.unwrap()?;

        Ok(())
    }
}
