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
#[async_trait::async_trait]
pub trait QuestionRepo: Send + Sync {
    async fn save(&self, question: &Question) -> Result<(), Error>;
    async fn delete(&self, question_id: Uuid) -> Result<(), Error>;
    async fn read(&self, question_id: Uuid) -> Result<Question, Error>;
    async fn update(&self, question: &Question) -> Result<(), Error>;
}
