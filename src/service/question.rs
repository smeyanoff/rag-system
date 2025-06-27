use std::fmt::Error;
use std::sync::Arc;

use crate::domain::embedding::{QuestionEmbeddingRepo, QuestionEmbending, TextVectorizer};
use crate::domain::question::{QuestionRepo, Question};

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
    pub fn process_new_question(&self, text: &str) -> Result<(), Error> {
        let question = Question::new(text.to_string());
        self.question_repo.save(&question)?;
        let question_embedding = QuestionEmbending::new(&question, self.vectorizer.as_ref())?;
        self.embedding_repo.save(&question_embedding)?;
        Ok(())
    }
}
