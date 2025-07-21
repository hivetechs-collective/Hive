// Consensus pipeline stages
// Each stage implements specific logic matching TypeScript behavior

pub mod curator;
pub mod enhanced_generator;
pub mod file_aware_generator;
pub mod file_aware_curator;
pub mod generator;
pub mod refiner;
pub mod validator;
pub mod repository_scanner;
pub mod claude_code_curator;

pub use curator::CuratorStage;
pub use file_aware_curator::FileAwareCuratorStage;
pub use generator::GeneratorStage;
pub use refiner::RefinerStage;
pub use validator::ValidatorStage;
pub use claude_code_curator::ClaudeCodeCuratorStage;

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
