mod bash;
mod pwsh;
mod zsh;

pub use bash::BashHook;
pub use pwsh::PwshHook;
pub use zsh::ZshHook;

/// Trait for shell integration hooks
pub trait ShellHook {
    /// Shell name
    fn name(&self) -> &str;

    /// Generate hook script
    fn generate(&self) -> String;

    /// Check if input should be intercepted
    fn should_intercept(&self, input: &str, trigger: &str) -> bool;
}
