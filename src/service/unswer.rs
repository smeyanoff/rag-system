use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{
    question::QuestionRepo,
    document::ChunkRepo,
    embedding::{QuestionEmbeddingRepo, VectorSearcher},
    unswer::{LLM, Unswer, UnswerRepo},
};

pub struct UnswerService {
    llm: Arc<dyn LLM>,
    unswer_repo: Arc<dyn UnswerRepo>,
    question_repo: Arc<dyn QuestionRepo>,
    question_embeding_repo: Arc<dyn QuestionEmbeddingRepo>,
    vector_searcher: Arc<dyn VectorSearcher>,
    chunk_repo: Arc<dyn ChunkRepo>,
}

impl UnswerService {
    pub fn new(
        llm: Arc<dyn LLM>,
        unswer_repo: Arc<dyn UnswerRepo>,
        question_repo: Arc<dyn QuestionRepo>,
        question_embeding_repo: Arc<dyn QuestionEmbeddingRepo>,
        vector_searcher: Arc<dyn VectorSearcher>,
        chunk_repo: Arc<dyn ChunkRepo>,
    ) -> Self {
        Self {
            llm,
            unswer_repo,
            question_repo,
            question_embeding_repo,
            vector_searcher,
            chunk_repo,
        }
    }
}

impl UnswerService {
    pub fn get_unswer(&self, question_id: Uuid) -> Result<String, std::fmt::Error> {
        let question = self.question_repo.read(question_id)?;
        let question_embedding = self.question_embeding_repo.read(question_id)?;
        let k_nearest = self
            .vector_searcher
            .search_similar(&question_embedding.vec, 5)?;
        let mut context = Vec::<String>::new();
        for chunk_id in k_nearest.clone() {
            let chunk = self.chunk_repo.read(chunk_id)?;
            context.push(chunk.text);
        }
        let unswer_text = self.llm.formulate_unswer(question.text, context)?;
        let unswer = Unswer::new(unswer_text, k_nearest);
        self.unswer_repo.save(&unswer)?;
        Ok(unswer.text)
    }
}
