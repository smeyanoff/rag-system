use std::fmt::Error;

use uuid::Uuid;

use crate::domain::{document::Chunk, question::Question};

pub struct ChunkEmbending {
    pub id: Uuid,
    pub chunk_id: Uuid,
    pub vec: Vec<f64>,
}

impl ChunkEmbending {
    pub async fn new(
        chunk: &Chunk,
        vectorizer: &dyn TextVectorizer,
    ) -> Result<ChunkEmbending, Error> {
        match vectorizer.vectorize(chunk.text.as_str()).await {
            Ok(vec) => Ok(Self {
                id: Uuid::new_v4(),
                chunk_id: chunk.id,
                vec: vec,
            }),
            Err(err) => Err(err),
        }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait TextVectorizer: Send + Sync {
    async fn vectorize(&self, text: &str) -> Result<Vec<f64>, Error>;
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait VectorSearcher: Send + Sync {
    async fn search_similar(&self, vector: &Vec<f64>, top_k: usize) -> Result<Vec<Uuid>, Error>;
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait ChunkEmbendingRepo: Send + Sync {
    async fn save(&self, embedding: &ChunkEmbending) -> Result<(), Error>;
    async fn delete(&self, chunk_id: Uuid) -> Result<(), Error>;
    async fn read(&self, chunk_id: Uuid) -> Result<ChunkEmbending, Error>;
}

pub struct QuestionEmbending {
    pub id: Uuid,
    pub question_id: Uuid,
    pub vec: Vec<f64>,
}

impl QuestionEmbending {
    pub async fn new(
        question: &Question,
        vectorizer: &dyn TextVectorizer,
    ) -> Result<QuestionEmbending, Error> {
        match vectorizer.vectorize(question.text.as_str()).await {
            Ok(vec) => Ok(Self {
                id: Uuid::new_v4(),
                question_id: question.id,
                vec: vec,
            }),
            Err(err) => Err(err),
        }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait QuestionEmbeddingRepo: Send + Sync {
    async fn save(&self, embedding: &QuestionEmbending) -> Result<(), Error>;
    async fn delete(&self, question_id: Uuid) -> Result<(), Error>;
    async fn read(&self, question_id: Uuid) -> Result<QuestionEmbending, Error>;
}
