use std::fmt::Error;

use uuid::Uuid;

use crate::domain::Chunk;

pub struct ChunkEmbending {
    pub id: Uuid,
    pub chunk_id: Uuid,
    pub vec: Vec<f64>,
}

impl ChunkEmbending {
    pub fn new(chunk: &Chunk, vectorizer: &dyn TextVectorizer) -> Result<ChunkEmbending, Error> {
        match vectorizer.vectorize(chunk.text.as_str()) {
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
pub trait TextVectorizer {
    fn vectorize(&self, text: &str) -> Result<Vec<f64>, Error>;
}

#[mockall::automock]
pub trait VectorSearcher {
    fn search_similar(&self, vector: &Vec<f64>, top_k: usize) -> Result<Vec<Uuid>, Error>;
}

#[mockall::automock]
pub trait ChunkEmbendingRepo {
    fn save(&self, embedding: &ChunkEmbending) -> Result<(), Error>;
    fn delete(&self, chunk_id: Uuid) -> Result<(), Error>;
    fn read(&self, chunk_id: Uuid) -> Result<ChunkEmbending, Error>;
}

pub struct QuestionEmbending {
    pub id: Uuid,
    pub question_id: Uuid,
    pub vec: Vec<f64>,
}
