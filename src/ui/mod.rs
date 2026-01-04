mod display;
mod hunk_display;
mod prompt;

pub use display::display_diff;
pub use hunk_display::{display_hunk, prompt_for_hunk_choice, HunkUserChoice};
pub use prompt::{prompt_for_action, UserChoice};
