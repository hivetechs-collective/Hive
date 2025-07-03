// Consensus pipeline stages
// Each stage implements specific logic matching TypeScript behavior

pub mod generator;
pub mod refiner;
pub mod validator;
pub mod curator;

pub use generator::GeneratorStage;
pub use refiner::RefinerStage; 
pub use validator::ValidatorStage;
pub use curator::CuratorStage;

use crate::consensus::types::{Message, Stage};
use anyhow::Result;

/// Common trait for all consensus stages
pub trait ConsensusStage: Send + Sync {
    /// Get the stage type
    fn stage(&self) -> Stage;

    /// Build messages for this stage
    fn build_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>>;

    /// Get the system prompt for this stage
    fn system_prompt(&self) -> &'static str;

    /// Process the stage (can be overridden for custom logic)
    fn process(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        self.build_messages(question, previous_answer, context)
    }
}