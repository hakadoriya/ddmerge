mod directory;
pub mod file;
pub mod hunk;

pub use directory::{compare_directories, DiffEntry, DiffType};
pub use file::{compare_files, read_text_file};
pub use hunk::{apply_hunk_choices, extract_hunks, Hunk, HunkChoice};
