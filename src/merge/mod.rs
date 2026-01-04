mod strategy;

pub use strategy::{
    apply_file_action, apply_hunk_merge, copy_unchanged_files, perform_merge, FileAction,
    MergeAction,
};
