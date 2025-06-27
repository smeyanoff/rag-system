use std::fmt::Error;

use uuid::Uuid;

pub struct Unswer {
    pub id: Uuid,
    pub text: String,
    pub context_chunks_id: Vec<Uuid>,
}

impl Unswer {
    pub fn new(text: String, context: Vec<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text,
            context_chunks_id: context,
        }
    }
}

#[mockall::automock]
pub trait UnswerRepo {
    fn save(&self, unswer: &Unswer) -> Result<(), Error>;
    fn read(&self, unswer_id: Uuid) -> Result<Unswer, Error>;
    fn delete(&self, unswer_id: Uuid) -> Result<(), Error>;
    fn update(&self, unswer: &Unswer) -> Result<(), Error>;
}

#[mockall::automock]
pub trait LLM {
    fn formulate_unswer(&self, question: String, context: Vec<String>) -> Result<String, Error>;
}
