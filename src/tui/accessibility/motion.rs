//! Motion and animation preferences
//!
//! Provides reduced motion support for users with vestibular disorders

/// Motion preferences manager
pub struct MotionPreferences {
    reduce_motion: bool,
    animation_speed: AnimationSpeed,
    disable_parallax: bool,
    disable_transitions: bool,
}

/// Animation speed settings
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationSpeed {
    /// No animations
    None,
    /// Very slow animations
    Slow,
    /// Normal speed animations
    Normal,
    /// Fast animations
    Fast,
}

impl MotionPreferences {
    /// Create new motion preferences manager
    pub fn new() -> Self {
        Self {
            reduce_motion: false,
            animation_speed: AnimationSpeed::Normal,
            disable_parallax: false,
            disable_transitions: false,
        }
    }

    /// Set reduce motion preference
    pub fn set_reduce_motion(&mut self, reduce: bool) {
        self.reduce_motion = reduce;
        
        if reduce {
            self.animation_speed = AnimationSpeed::None;
            self.disable_parallax = true;
            self.disable_transitions = true;
        }
    }

    /// Check if motion should be reduced
    pub fn should_reduce_motion(&self) -> bool {
        self.reduce_motion
    }

    /// Set animation speed
    pub fn set_animation_speed(&mut self, speed: AnimationSpeed) {
        self.animation_speed = speed;
    }

    /// Get current animation speed
    pub fn animation_speed(&self) -> &AnimationSpeed {
        &self.animation_speed
    }

    /// Check if parallax effects should be disabled
    pub fn disable_parallax(&self) -> bool {
        self.disable_parallax
    }

    /// Check if transitions should be disabled
    pub fn disable_transitions(&self) -> bool {
        self.disable_transitions
    }

    /// Get animation duration multiplier
    pub fn animation_duration_multiplier(&self) -> f32 {
        match self.animation_speed {
            AnimationSpeed::None => 0.0,
            AnimationSpeed::Slow => 2.0,
            AnimationSpeed::Normal => 1.0,
            AnimationSpeed::Fast => 0.5,
        }
    }

    /// Check if animation should be played
    pub fn should_animate(&self, animation_type: AnimationType) -> bool {
        if self.reduce_motion {
            return false;
        }

        match animation_type {
            AnimationType::Scroll => !self.reduce_motion,
            AnimationType::Fade => !self.disable_transitions,
            AnimationType::Slide => !self.disable_transitions,
            AnimationType::Parallax => !self.disable_parallax,
            AnimationType::Spin => false, // Always disable spinning animations
            AnimationType::Bounce => self.animation_speed != AnimationSpeed::None,
            AnimationType::Instant => true,  // Always allow instant animations
            AnimationType::Static => true,   // Always allow static (no animation)
            AnimationType::Jump => !self.reduce_motion,
        }
    }

    /// Get safe animation alternatives
    pub fn get_safe_alternative(&self, animation_type: AnimationType) -> AnimationType {
        if self.should_animate(animation_type.clone()) {
            animation_type
        } else {
            match animation_type {
                AnimationType::Fade | AnimationType::Slide => AnimationType::Instant,
                AnimationType::Spin | AnimationType::Bounce => AnimationType::Static,
                AnimationType::Parallax => AnimationType::Static,
                AnimationType::Scroll => AnimationType::Jump,
                _ => AnimationType::Static,
            }
        }
    }
}

/// Types of animations
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationType {
    /// Scrolling animations
    Scroll,
    /// Fade in/out effects
    Fade,
    /// Sliding transitions
    Slide,
    /// Parallax scrolling effects
    Parallax,
    /// Spinning/rotating animations
    Spin,
    /// Bouncing effects
    Bounce,
    /// Instant appearance (no animation)
    Instant,
    /// Static display (no movement)
    Static,
    /// Jump to position (no smooth scrolling)
    Jump,
}

impl Default for MotionPreferences {
    fn default() -> Self {
        Self::new()
    }
}