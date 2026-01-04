use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::diff::{DiffEntry, DiffType};

/// Action to take for a file-level diff entry (LeftOnly/RightOnly)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    /// Copy to the other directory
    Copy,
    /// Delete from the source directory
    Delete,
    /// Skip (leave as is)
    Skip,
}

/// Apply file-level action for LeftOnly/RightOnly entries
pub fn apply_file_action(
    entry: &DiffEntry,
    action: FileAction,
    left_root: &Path,
    right_root: &Path,
) -> Result<()> {
    match (&entry.diff_type, action) {
        // LeftOnly: file exists only in left
        (DiffType::LeftOnly, FileAction::Copy) => {
            // Copy from left to right
            let src = left_root.join(&entry.path);
            let dst = right_root.join(&entry.path);
            copy_entry(&src, &dst)?;
        }
        (DiffType::LeftOnly, FileAction::Delete) => {
            // Delete from left
            let path = left_root.join(&entry.path);
            remove_entry(&path)?;
        }
        (DiffType::LeftOnly, FileAction::Skip) => {
            // Do nothing
        }

        // RightOnly: file exists only in right
        (DiffType::RightOnly, FileAction::Copy) => {
            // Copy from right to left
            let src = right_root.join(&entry.path);
            let dst = left_root.join(&entry.path);
            copy_entry(&src, &dst)?;
        }
        (DiffType::RightOnly, FileAction::Delete) => {
            // Delete from right
            let path = right_root.join(&entry.path);
            remove_entry(&path)?;
        }
        (DiffType::RightOnly, FileAction::Skip) => {
            // Do nothing
        }

        // TypeMismatch: same name but different types
        (DiffType::TypeMismatch, FileAction::Copy) => {
            // This is ambiguous - for now, copy left to right
            let src = left_root.join(&entry.path);
            let dst = right_root.join(&entry.path);
            remove_entry(&dst)?;
            copy_entry(&src, &dst)?;
        }
        (DiffType::TypeMismatch, FileAction::Delete) => {
            // Delete both? Or just one? For now, delete from right
            let path = right_root.join(&entry.path);
            remove_entry(&path)?;
        }
        (DiffType::TypeMismatch, FileAction::Skip) => {
            // Do nothing
        }

        _ => {
            // Modified files should use hunk-based merge
        }
    }

    Ok(())
}

/// Apply hunk choices to merge a modified file
/// Updates left file with left_content and right file with right_content
pub fn apply_hunk_merge(
    left_path: &Path,
    right_path: &Path,
    left_content: &str,
    right_content: &str,
) -> Result<()> {
    // Write merged content to both files
    fs::write(left_path, left_content)?;
    fs::write(right_path, right_content)?;
    Ok(())
}

/// Copy a file or directory recursively
fn copy_entry(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    if src.is_dir() {
        copy_dir_all(src, dst)?;
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

/// Remove a file or directory
fn remove_entry(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

// Keep old types for backwards compatibility during transition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeAction {
    UseLeft,
    UseRight,
    Keep,
    Delete,
    Skip,
}

pub fn perform_merge(
    _entry: &DiffEntry,
    _action: MergeAction,
    _left_root: &Path,
    _right_root: &Path,
    _output_root: &Path,
) -> Result<()> {
    // Deprecated - use apply_file_action or apply_hunk_merge instead
    Ok(())
}

pub fn copy_unchanged_files(
    _left_root: &Path,
    _right_root: &Path,
    _output_root: &Path,
    _diffs: &[DiffEntry],
) -> Result<()> {
    // Deprecated - no longer needed for in-place merge
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_dirs() -> (TempDir, TempDir) {
        let left = TempDir::new().unwrap();
        let right = TempDir::new().unwrap();
        (left, right)
    }

    fn create_diff_entry(path: &str, diff_type: DiffType) -> DiffEntry {
        let (left_is_dir, right_is_dir) = match diff_type {
            DiffType::LeftOnly => (Some(false), None),
            DiffType::RightOnly => (None, Some(false)),
            DiffType::Modified => (Some(false), Some(false)),
            DiffType::TypeMismatch => (Some(false), Some(true)),
        };
        DiffEntry {
            path: PathBuf::from(path),
            diff_type,
            left_is_dir,
            right_is_dir,
        }
    }

    fn create_diff_entry_with_types(
        path: &str,
        diff_type: DiffType,
        left_is_dir: Option<bool>,
        right_is_dir: Option<bool>,
    ) -> DiffEntry {
        DiffEntry {
            path: PathBuf::from(path),
            diff_type,
            left_is_dir,
            right_is_dir,
        }
    }

    // ========================================
    // apply_file_action tests - LeftOnly
    // ========================================

    #[test]
    fn test_apply_file_action_left_only_copy() {
        // Given: A file exists only in the left directory
        let (left, right) = create_test_dirs();
        let file_path = "test.txt";
        fs::write(left.path().join(file_path), "left content").unwrap();

        let entry = create_diff_entry(file_path, DiffType::LeftOnly);

        // When: Copy action is applied
        apply_file_action(&entry, FileAction::Copy, left.path(), right.path()).unwrap();

        // Then: The file is copied to the right directory
        assert!(right.path().join(file_path).exists());
        assert_eq!(
            fs::read_to_string(right.path().join(file_path)).unwrap(),
            "left content"
        );
    }

    #[test]
    fn test_apply_file_action_left_only_copy_nested() {
        // Given: A file in a nested directory exists only in the left directory
        let (left, right) = create_test_dirs();
        let file_path = "subdir/nested/test.txt";
        fs::create_dir_all(left.path().join("subdir/nested")).unwrap();
        fs::write(left.path().join(file_path), "nested content").unwrap();

        let entry = create_diff_entry(file_path, DiffType::LeftOnly);

        // When: Copy action is applied
        apply_file_action(&entry, FileAction::Copy, left.path(), right.path()).unwrap();

        // Then: The file and parent directories are created in the right directory
        assert!(right.path().join(file_path).exists());
        assert_eq!(
            fs::read_to_string(right.path().join(file_path)).unwrap(),
            "nested content"
        );
    }

    #[test]
    fn test_apply_file_action_left_only_delete() {
        // Given: A file exists only in the left directory
        let (left, right) = create_test_dirs();
        let file_path = "test.txt";
        fs::write(left.path().join(file_path), "left content").unwrap();

        let entry = create_diff_entry(file_path, DiffType::LeftOnly);

        // When: Delete action is applied
        apply_file_action(&entry, FileAction::Delete, left.path(), right.path()).unwrap();

        // Then: The file is deleted from the left directory
        assert!(!left.path().join(file_path).exists());
    }

    #[test]
    fn test_apply_file_action_left_only_skip() {
        // Given: A file exists only in the left directory
        let (left, right) = create_test_dirs();
        let file_path = "test.txt";
        fs::write(left.path().join(file_path), "left content").unwrap();

        let entry = create_diff_entry(file_path, DiffType::LeftOnly);

        // When: Skip action is applied
        apply_file_action(&entry, FileAction::Skip, left.path(), right.path()).unwrap();

        // Then: The file remains in the left directory and is not copied to right
        assert!(left.path().join(file_path).exists());
        assert!(!right.path().join(file_path).exists());
    }

    // ========================================
    // apply_file_action tests - RightOnly
    // ========================================

    #[test]
    fn test_apply_file_action_right_only_copy() {
        // Given: A file exists only in the right directory
        let (left, right) = create_test_dirs();
        let file_path = "test.txt";
        fs::write(right.path().join(file_path), "right content").unwrap();

        let entry = create_diff_entry(file_path, DiffType::RightOnly);

        // When: Copy action is applied
        apply_file_action(&entry, FileAction::Copy, left.path(), right.path()).unwrap();

        // Then: The file is copied to the left directory
        assert!(left.path().join(file_path).exists());
        assert_eq!(
            fs::read_to_string(left.path().join(file_path)).unwrap(),
            "right content"
        );
    }

    #[test]
    fn test_apply_file_action_right_only_delete() {
        // Given: A file exists only in the right directory
        let (left, right) = create_test_dirs();
        let file_path = "test.txt";
        fs::write(right.path().join(file_path), "right content").unwrap();

        let entry = create_diff_entry(file_path, DiffType::RightOnly);

        // When: Delete action is applied
        apply_file_action(&entry, FileAction::Delete, left.path(), right.path()).unwrap();

        // Then: The file is deleted from the right directory
        assert!(!right.path().join(file_path).exists());
    }

    #[test]
    fn test_apply_file_action_right_only_skip() {
        // Given: A file exists only in the right directory
        let (left, right) = create_test_dirs();
        let file_path = "test.txt";
        fs::write(right.path().join(file_path), "right content").unwrap();

        let entry = create_diff_entry(file_path, DiffType::RightOnly);

        // When: Skip action is applied
        apply_file_action(&entry, FileAction::Skip, left.path(), right.path()).unwrap();

        // Then: The file remains in the right directory and is not copied to left
        assert!(right.path().join(file_path).exists());
        assert!(!left.path().join(file_path).exists());
    }

    // ========================================
    // apply_file_action tests - TypeMismatch
    // ========================================

    #[test]
    fn test_apply_file_action_type_mismatch_copy_file_over_dir() {
        // Given: Left has a file, right has a directory with the same name
        let (left, right) = create_test_dirs();
        let name = "item";
        fs::write(left.path().join(name), "file content").unwrap();
        fs::create_dir(right.path().join(name)).unwrap();

        let entry = create_diff_entry(name, DiffType::TypeMismatch);

        // When: Copy action is applied
        apply_file_action(&entry, FileAction::Copy, left.path(), right.path()).unwrap();

        // Then: The right directory is replaced with the left file
        assert!(right.path().join(name).is_file());
        assert_eq!(
            fs::read_to_string(right.path().join(name)).unwrap(),
            "file content"
        );
    }

    #[test]
    fn test_apply_file_action_type_mismatch_copy_dir_over_file() {
        // Given: Left has a directory, right has a file with the same name
        let (left, right) = create_test_dirs();
        let name = "item";
        fs::create_dir(left.path().join(name)).unwrap();
        fs::write(left.path().join(name).join("child.txt"), "child").unwrap();
        fs::write(right.path().join(name), "file content").unwrap();

        let entry = create_diff_entry(name, DiffType::TypeMismatch);

        // When: Copy action is applied
        apply_file_action(&entry, FileAction::Copy, left.path(), right.path()).unwrap();

        // Then: The right file is replaced with the left directory
        assert!(right.path().join(name).is_dir());
        assert!(right.path().join(name).join("child.txt").exists());
    }

    #[test]
    fn test_apply_file_action_type_mismatch_delete() {
        // Given: Left has a file, right has a directory with the same name
        let (left, right) = create_test_dirs();
        let name = "item";
        fs::write(left.path().join(name), "file content").unwrap();
        fs::create_dir(right.path().join(name)).unwrap();

        let entry = create_diff_entry(name, DiffType::TypeMismatch);

        // When: Delete action is applied
        apply_file_action(&entry, FileAction::Delete, left.path(), right.path()).unwrap();

        // Then: The right item is deleted, left remains
        assert!(!right.path().join(name).exists());
        assert!(left.path().join(name).exists());
    }

    #[test]
    fn test_apply_file_action_type_mismatch_skip() {
        // Given: Left has a file, right has a directory with the same name
        let (left, right) = create_test_dirs();
        let name = "item";
        fs::write(left.path().join(name), "file content").unwrap();
        fs::create_dir(right.path().join(name)).unwrap();

        let entry = create_diff_entry(name, DiffType::TypeMismatch);

        // When: Skip action is applied
        apply_file_action(&entry, FileAction::Skip, left.path(), right.path()).unwrap();

        // Then: Both items remain unchanged
        assert!(left.path().join(name).is_file());
        assert!(right.path().join(name).is_dir());
    }

    // ========================================
    // apply_file_action tests - Modified
    // ========================================

    #[test]
    fn test_apply_file_action_modified_does_nothing() {
        // Given: A modified file exists in both directories
        let (left, right) = create_test_dirs();
        let file_path = "test.txt";
        fs::write(left.path().join(file_path), "left content").unwrap();
        fs::write(right.path().join(file_path), "right content").unwrap();

        let entry = create_diff_entry(file_path, DiffType::Modified);

        // When: Any action is applied to a Modified entry
        apply_file_action(&entry, FileAction::Copy, left.path(), right.path()).unwrap();

        // Then: Both files remain unchanged (Modified uses hunk-based merge)
        assert_eq!(
            fs::read_to_string(left.path().join(file_path)).unwrap(),
            "left content"
        );
        assert_eq!(
            fs::read_to_string(right.path().join(file_path)).unwrap(),
            "right content"
        );
    }

    // ========================================
    // apply_file_action tests - Directory operations
    // ========================================

    #[test]
    fn test_apply_file_action_copy_directory() {
        // Given: A directory with files exists only in the left directory
        let (left, right) = create_test_dirs();
        let dir_path = "test-dir";
        fs::create_dir(left.path().join(dir_path)).unwrap();
        fs::write(left.path().join(dir_path).join("file1.txt"), "content1").unwrap();
        fs::write(left.path().join(dir_path).join("file2.txt"), "content2").unwrap();

        let entry = create_diff_entry_with_types(dir_path, DiffType::LeftOnly, Some(true), None);

        // When: Copy action is applied
        apply_file_action(&entry, FileAction::Copy, left.path(), right.path()).unwrap();

        // Then: The entire directory structure is copied to the right
        assert!(right.path().join(dir_path).is_dir());
        assert_eq!(
            fs::read_to_string(right.path().join(dir_path).join("file1.txt")).unwrap(),
            "content1"
        );
        assert_eq!(
            fs::read_to_string(right.path().join(dir_path).join("file2.txt")).unwrap(),
            "content2"
        );
    }

    #[test]
    fn test_apply_file_action_copy_nested_directory() {
        // Given: A nested directory structure exists only in the left directory
        let (left, right) = create_test_dirs();
        let dir_path = "parent/child/grandchild";
        fs::create_dir_all(left.path().join(dir_path)).unwrap();
        fs::write(left.path().join(dir_path).join("deep.txt"), "deep content").unwrap();

        let entry = create_diff_entry_with_types("parent", DiffType::LeftOnly, Some(true), None);

        // When: Copy action is applied
        apply_file_action(&entry, FileAction::Copy, left.path(), right.path()).unwrap();

        // Then: The entire nested structure is copied
        assert!(right.path().join(dir_path).is_dir());
        assert_eq!(
            fs::read_to_string(right.path().join(dir_path).join("deep.txt")).unwrap(),
            "deep content"
        );
    }

    #[test]
    fn test_apply_file_action_delete_directory() {
        // Given: A directory with files exists only in the left directory
        let (left, right) = create_test_dirs();
        let dir_path = "test-dir";
        fs::create_dir(left.path().join(dir_path)).unwrap();
        fs::write(left.path().join(dir_path).join("file.txt"), "content").unwrap();

        let entry = create_diff_entry_with_types(dir_path, DiffType::LeftOnly, Some(true), None);

        // When: Delete action is applied
        apply_file_action(&entry, FileAction::Delete, left.path(), right.path()).unwrap();

        // Then: The entire directory is deleted from left
        assert!(!left.path().join(dir_path).exists());
    }

    // ========================================
    // apply_hunk_merge tests
    // ========================================

    #[test]
    fn test_apply_hunk_merge_writes_both_files() {
        // Given: Two existing files with different content
        let (left, right) = create_test_dirs();
        let left_path = left.path().join("test.txt");
        let right_path = right.path().join("test.txt");
        fs::write(&left_path, "old left").unwrap();
        fs::write(&right_path, "old right").unwrap();

        // When: apply_hunk_merge is called with new content
        apply_hunk_merge(
            &left_path,
            &right_path,
            "new left content",
            "new right content",
        )
        .unwrap();

        // Then: Both files are updated with the new content
        assert_eq!(fs::read_to_string(&left_path).unwrap(), "new left content");
        assert_eq!(
            fs::read_to_string(&right_path).unwrap(),
            "new right content"
        );
    }

    #[test]
    fn test_apply_hunk_merge_creates_files_if_not_exist() {
        // Given: Target paths where no files exist yet
        let (left, right) = create_test_dirs();
        let left_path = left.path().join("new.txt");
        let right_path = right.path().join("new.txt");

        // When: apply_hunk_merge is called
        apply_hunk_merge(&left_path, &right_path, "left content", "right content").unwrap();

        // Then: Both files are created with the specified content
        assert_eq!(fs::read_to_string(&left_path).unwrap(), "left content");
        assert_eq!(fs::read_to_string(&right_path).unwrap(), "right content");
    }

    #[test]
    fn test_apply_hunk_merge_same_content_syncs_files() {
        // Given: Two files that need to be synchronized
        let (left, right) = create_test_dirs();
        let left_path = left.path().join("test.txt");
        let right_path = right.path().join("test.txt");
        fs::write(&left_path, "different").unwrap();
        fs::write(&right_path, "content").unwrap();

        // When: apply_hunk_merge is called with the same content for both
        let synced_content = "synced content\nline2\n";
        apply_hunk_merge(&left_path, &right_path, synced_content, synced_content).unwrap();

        // Then: Both files have identical content
        assert_eq!(fs::read_to_string(&left_path).unwrap(), synced_content);
        assert_eq!(fs::read_to_string(&right_path).unwrap(), synced_content);
    }

    #[test]
    fn test_apply_hunk_merge_preserves_trailing_newline() {
        // Given: Content with specific trailing newline behavior
        let (left, right) = create_test_dirs();
        let left_path = left.path().join("test.txt");
        let right_path = right.path().join("test.txt");

        // When: apply_hunk_merge is called with content without trailing newline
        apply_hunk_merge(&left_path, &right_path, "no newline", "has newline\n").unwrap();

        // Then: The exact content is preserved including trailing newline differences
        assert_eq!(fs::read_to_string(&left_path).unwrap(), "no newline");
        assert_eq!(fs::read_to_string(&right_path).unwrap(), "has newline\n");
    }
}
