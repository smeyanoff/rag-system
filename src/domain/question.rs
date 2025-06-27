use std::fmt::Error;

use uuid::Uuid;

pub struct Question {
    pub id: Uuid,
    pub text: String,
}

impl Question {
    pub fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text,
        }
    }
}

#[mockall::automock]
pub trait QuestionRepo {
    fn save(&self, question: &Question) -> Result<(), Error>;
    fn delete(&self, question_id: Uuid) -> Result<(), Error>;
    fn read(&self, question_id: Uuid) -> Result<Question, Error>;
    fn update(&self, question: &Question) -> Result<(), Error>;
}
