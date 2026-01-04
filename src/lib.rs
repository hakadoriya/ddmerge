pub mod diff;
pub mod merge;
pub mod ui;

pub use diff::{
    apply_hunk_choices, compare_directories, extract_hunks, DiffEntry, DiffType, Hunk, HunkChoice,
};
pub use merge::{apply_file_action, apply_hunk_merge, FileAction};
