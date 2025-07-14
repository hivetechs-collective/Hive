//! Consensus Types for TUI Display

/// Simple consensus progress state for inline display
#[derive(Clone, Debug)]
pub enum SimpleConsensusProgress {
    Thinking,
    Generating(u8),
    Refining(u8),
    Validating(u8),
    Curating(u8),
    Complete,
    Error(String),
}

impl SimpleConsensusProgress {
    /// Convert from detailed ConsensusProgress to simple enum
    pub fn from_detailed(progress: &crate::tui::consensus_view::ConsensusProgress) -> Self {
        use crate::tui::consensus_view::StageStatus;

        if !progress.is_active {
            return SimpleConsensusProgress::Complete;
        }

        if progress.generator.status == StageStatus::Running {
            SimpleConsensusProgress::Generating(progress.generator.progress as u8)
        } else if progress.refiner.status == StageStatus::Running {
            SimpleConsensusProgress::Refining(progress.refiner.progress as u8)
        } else if progress.validator.status == StageStatus::Running {
            SimpleConsensusProgress::Validating(progress.validator.progress as u8)
        } else if progress.curator.status == StageStatus::Running {
            SimpleConsensusProgress::Curating(progress.curator.progress as u8)
        } else if progress.curator.status == StageStatus::Completed {
            SimpleConsensusProgress::Complete
        } else if progress.generator.status == StageStatus::Error
            || progress.refiner.status == StageStatus::Error
            || progress.validator.status == StageStatus::Error
            || progress.curator.status == StageStatus::Error
        {
            SimpleConsensusProgress::Error("Pipeline error".to_string())
        } else {
            SimpleConsensusProgress::Thinking
        }
    }
}
