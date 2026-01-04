use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::file::compare_files;

/// Type of difference between two directories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffType {
    /// File or directory exists only in the left directory
    LeftOnly,
    /// File or directory exists only in the right directory
    RightOnly,
    /// File exists in both but content differs
    Modified,
    /// Same path but different types (file vs directory)
    TypeMismatch,
}

/// A single difference entry
#[derive(Debug, Clone)]
pub struct DiffEntry {
    /// Relative path from the root directory
    pub path: PathBuf,
    /// Type of difference
    pub diff_type: DiffType,
    /// Whether left side is a directory (if exists)
    pub left_is_dir: Option<bool>,
    /// Whether right side is a directory (if exists)
    pub right_is_dir: Option<bool>,
}

impl DiffEntry {
    pub fn left_only(path: PathBuf, is_dir: bool) -> Self {
        Self {
            path,
            diff_type: DiffType::LeftOnly,
            left_is_dir: Some(is_dir),
            right_is_dir: None,
        }
    }

    pub fn right_only(path: PathBuf, is_dir: bool) -> Self {
        Self {
            path,
            diff_type: DiffType::RightOnly,
            left_is_dir: None,
            right_is_dir: Some(is_dir),
        }
    }

    pub fn modified(path: PathBuf) -> Self {
        Self {
            path,
            diff_type: DiffType::Modified,
            left_is_dir: Some(false),
            right_is_dir: Some(false),
        }
    }

    pub fn type_mismatch(path: PathBuf, left_is_dir: bool, right_is_dir: bool) -> Self {
        Self {
            path,
            diff_type: DiffType::TypeMismatch,
            left_is_dir: Some(left_is_dir),
            right_is_dir: Some(right_is_dir),
        }
    }
}

/// Collect all relative paths from a directory
fn collect_paths(root: &Path) -> Result<BTreeSet<PathBuf>> {
    let mut paths = BTreeSet::new();

    for entry in WalkDir::new(root).min_depth(1) {
        let entry = entry?;
        let rel_path = entry.path().strip_prefix(root)?.to_path_buf();
        paths.insert(rel_path);
    }

    Ok(paths)
}

/// Compare two directories and return all differences
pub fn compare_directories(left: &Path, right: &Path) -> Result<Vec<DiffEntry>> {
    let left_paths = collect_paths(left)?;
    let right_paths = collect_paths(right)?;

    let mut diffs = Vec::new();

    // Find all unique paths
    let all_paths: BTreeSet<_> = left_paths.union(&right_paths).cloned().collect();

    for rel_path in all_paths {
        let left_full = left.join(&rel_path);
        let right_full = right.join(&rel_path);

        let left_exists = left_full.exists();
        let right_exists = right_full.exists();

        match (left_exists, right_exists) {
            (true, false) => {
                let is_dir = left_full.is_dir();
                // Skip directory contents if parent directory is already marked as LeftOnly
                if !is_dir || !has_parent_diff(&diffs, &rel_path, DiffType::LeftOnly) {
                    diffs.push(DiffEntry::left_only(rel_path, is_dir));
                }
            }
            (false, true) => {
                let is_dir = right_full.is_dir();
                // Skip directory contents if parent directory is already marked as RightOnly
                if !is_dir || !has_parent_diff(&diffs, &rel_path, DiffType::RightOnly) {
                    diffs.push(DiffEntry::right_only(rel_path, is_dir));
                }
            }
            (true, true) => {
                let left_is_dir = left_full.is_dir();
                let right_is_dir = right_full.is_dir();

                if left_is_dir != right_is_dir {
                    diffs.push(DiffEntry::type_mismatch(
                        rel_path,
                        left_is_dir,
                        right_is_dir,
                    ));
                } else if !left_is_dir {
                    // Both are files, compare content
                    if !compare_files(&left_full, &right_full)? {
                        diffs.push(DiffEntry::modified(rel_path));
                    }
                }
                // If both are directories with same type, no diff for the directory itself
            }
            (false, false) => {
                // This shouldn't happen, but handle gracefully
            }
        }
    }

    // Filter out child entries when parent directory is LeftOnly or RightOnly
    let diffs = filter_nested_diffs(diffs);

    Ok(diffs)
}

/// Check if there's a parent directory with the given diff type
fn has_parent_diff(diffs: &[DiffEntry], path: &Path, diff_type: DiffType) -> bool {
    for ancestor in path.ancestors().skip(1) {
        if ancestor.as_os_str().is_empty() {
            break;
        }
        for diff in diffs {
            if diff.path == ancestor && diff.diff_type == diff_type {
                return true;
            }
        }
    }
    false
}

/// Filter out entries that are children of LeftOnly or RightOnly directories
fn filter_nested_diffs(diffs: Vec<DiffEntry>) -> Vec<DiffEntry> {
    let only_dirs: Vec<PathBuf> = diffs
        .iter()
        .filter(|d| {
            (d.diff_type == DiffType::LeftOnly || d.diff_type == DiffType::RightOnly)
                && (d.left_is_dir.unwrap_or(false) || d.right_is_dir.unwrap_or(false))
        })
        .map(|d| d.path.clone())
        .collect();

    diffs
        .into_iter()
        .filter(|d| {
            !only_dirs
                .iter()
                .any(|dir| d.path != *dir && d.path.starts_with(dir))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_dirs() -> (TempDir, TempDir) {
        let left = TempDir::new().unwrap();
        let right = TempDir::new().unwrap();
        (left, right)
    }

    // ========================================
    // compare_directories tests - Basic cases
    // ========================================

    #[test]
    fn test_identical_directories() {
        // Given: Two directories with identical files
        let (left, right) = setup_test_dirs();
        fs::write(left.path().join("file.txt"), "content").unwrap();
        fs::write(right.path().join("file.txt"), "content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: No differences are found
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_empty_directories() {
        // Given: Two empty directories
        let (left, right) = setup_test_dirs();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: No differences are found
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_left_only_file() {
        // Given: A file exists only in the left directory
        let (left, right) = setup_test_dirs();
        fs::write(left.path().join("only_left.txt"), "content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: One LeftOnly diff is found
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::LeftOnly);
        assert_eq!(diffs[0].path, PathBuf::from("only_left.txt"));
        assert_eq!(diffs[0].left_is_dir, Some(false));
        assert_eq!(diffs[0].right_is_dir, None);
    }

    #[test]
    fn test_right_only_file() {
        // Given: A file exists only in the right directory
        let (left, right) = setup_test_dirs();
        fs::write(right.path().join("only_right.txt"), "content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: One RightOnly diff is found
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::RightOnly);
        assert_eq!(diffs[0].path, PathBuf::from("only_right.txt"));
        assert_eq!(diffs[0].left_is_dir, None);
        assert_eq!(diffs[0].right_is_dir, Some(false));
    }

    #[test]
    fn test_modified_file() {
        // Given: A file exists in both directories with different content
        let (left, right) = setup_test_dirs();
        fs::write(left.path().join("file.txt"), "left content").unwrap();
        fs::write(right.path().join("file.txt"), "right content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: One Modified diff is found
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::Modified);
        assert_eq!(diffs[0].left_is_dir, Some(false));
        assert_eq!(diffs[0].right_is_dir, Some(false));
    }

    // ========================================
    // compare_directories tests - TypeMismatch
    // ========================================

    #[test]
    fn test_type_mismatch_file_vs_dir() {
        // Given: Left has a file, right has a directory with the same name
        let (left, right) = setup_test_dirs();
        fs::write(left.path().join("item"), "file content").unwrap();
        fs::create_dir(right.path().join("item")).unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: One TypeMismatch diff is found
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::TypeMismatch);
        assert_eq!(diffs[0].left_is_dir, Some(false));
        assert_eq!(diffs[0].right_is_dir, Some(true));
    }

    #[test]
    fn test_type_mismatch_dir_vs_file() {
        // Given: Left has a directory, right has a file with the same name
        let (left, right) = setup_test_dirs();
        fs::create_dir(left.path().join("item")).unwrap();
        fs::write(right.path().join("item"), "file content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: One TypeMismatch diff is found
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::TypeMismatch);
        assert_eq!(diffs[0].left_is_dir, Some(true));
        assert_eq!(diffs[0].right_is_dir, Some(false));
    }

    // ========================================
    // compare_directories tests - Nested directories
    // ========================================

    #[test]
    fn test_left_only_directory() {
        // Given: A directory with files exists only in the left
        let (left, right) = setup_test_dirs();
        fs::create_dir(left.path().join("subdir")).unwrap();
        fs::write(left.path().join("subdir/file.txt"), "content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: Only one LeftOnly diff for the directory (not its contents)
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::LeftOnly);
        assert_eq!(diffs[0].path, PathBuf::from("subdir"));
        assert_eq!(diffs[0].left_is_dir, Some(true));
    }

    #[test]
    fn test_right_only_directory() {
        // Given: A directory with files exists only in the right
        let (left, right) = setup_test_dirs();
        fs::create_dir(right.path().join("subdir")).unwrap();
        fs::write(right.path().join("subdir/file.txt"), "content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: Only one RightOnly diff for the directory (not its contents)
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::RightOnly);
        assert_eq!(diffs[0].path, PathBuf::from("subdir"));
        assert_eq!(diffs[0].right_is_dir, Some(true));
    }

    #[test]
    fn test_nested_directory_with_modified_file() {
        // Given: A shared nested directory with a modified file
        let (left, right) = setup_test_dirs();
        fs::create_dir(left.path().join("subdir")).unwrap();
        fs::create_dir(right.path().join("subdir")).unwrap();
        fs::write(left.path().join("subdir/file.txt"), "left").unwrap();
        fs::write(right.path().join("subdir/file.txt"), "right").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: One Modified diff for the file (directory itself has no diff)
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::Modified);
        assert_eq!(diffs[0].path, PathBuf::from("subdir/file.txt"));
    }

    #[test]
    fn test_deeply_nested_left_only() {
        // Given: A deeply nested directory structure only in left
        let (left, right) = setup_test_dirs();
        fs::create_dir_all(left.path().join("a/b/c")).unwrap();
        fs::write(left.path().join("a/b/c/file.txt"), "content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: Only one LeftOnly diff for the top-level directory
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::LeftOnly);
        assert_eq!(diffs[0].path, PathBuf::from("a"));
    }

    // ========================================
    // compare_directories tests - Multiple files
    // ========================================

    #[test]
    fn test_multiple_diff_types() {
        // Given: Multiple files with different diff types
        let (left, right) = setup_test_dirs();
        fs::write(left.path().join("left_only.txt"), "left").unwrap();
        fs::write(right.path().join("right_only.txt"), "right").unwrap();
        fs::write(left.path().join("modified.txt"), "left content").unwrap();
        fs::write(right.path().join("modified.txt"), "right content").unwrap();
        fs::write(left.path().join("same.txt"), "same").unwrap();
        fs::write(right.path().join("same.txt"), "same").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: Three diffs are found (left_only, right_only, modified)
        assert_eq!(diffs.len(), 3);
        let diff_types: Vec<_> = diffs.iter().map(|d| &d.diff_type).collect();
        assert!(diff_types.contains(&&DiffType::LeftOnly));
        assert!(diff_types.contains(&&DiffType::RightOnly));
        assert!(diff_types.contains(&&DiffType::Modified));
    }

    #[test]
    fn test_identical_directories_multiple_files() {
        // Given: Multiple identical files in both directories
        let (left, right) = setup_test_dirs();
        fs::write(left.path().join("a.txt"), "content a").unwrap();
        fs::write(right.path().join("a.txt"), "content a").unwrap();
        fs::write(left.path().join("b.txt"), "content b").unwrap();
        fs::write(right.path().join("b.txt"), "content b").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: No differences are found
        assert!(diffs.is_empty());
    }

    // ========================================
    // DiffEntry helper tests
    // ========================================

    #[test]
    fn test_diff_entry_left_only() {
        // Given: A path for a file that exists only in left
        let path = PathBuf::from("test.txt");

        // When: Creating a LeftOnly DiffEntry
        let entry = DiffEntry::left_only(path.clone(), false);

        // Then: The entry is correctly populated
        assert_eq!(entry.path, path);
        assert_eq!(entry.diff_type, DiffType::LeftOnly);
        assert_eq!(entry.left_is_dir, Some(false));
        assert_eq!(entry.right_is_dir, None);
    }

    #[test]
    fn test_diff_entry_left_only_dir() {
        // Given: A path for a directory that exists only in left
        let path = PathBuf::from("test-dir");

        // When: Creating a LeftOnly DiffEntry for a directory
        let entry = DiffEntry::left_only(path.clone(), true);

        // Then: The entry correctly marks it as a directory
        assert_eq!(entry.left_is_dir, Some(true));
    }

    #[test]
    fn test_diff_entry_right_only() {
        // Given: A path for a file that exists only in right
        let path = PathBuf::from("test.txt");

        // When: Creating a RightOnly DiffEntry
        let entry = DiffEntry::right_only(path.clone(), false);

        // Then: The entry is correctly populated
        assert_eq!(entry.path, path);
        assert_eq!(entry.diff_type, DiffType::RightOnly);
        assert_eq!(entry.left_is_dir, None);
        assert_eq!(entry.right_is_dir, Some(false));
    }

    #[test]
    fn test_diff_entry_modified() {
        // Given: A path for a modified file
        let path = PathBuf::from("test.txt");

        // When: Creating a Modified DiffEntry
        let entry = DiffEntry::modified(path.clone());

        // Then: The entry is correctly populated
        assert_eq!(entry.path, path);
        assert_eq!(entry.diff_type, DiffType::Modified);
        assert_eq!(entry.left_is_dir, Some(false));
        assert_eq!(entry.right_is_dir, Some(false));
    }

    #[test]
    fn test_diff_entry_type_mismatch() {
        // Given: A path with type mismatch (file on left, dir on right)
        let path = PathBuf::from("item");

        // When: Creating a TypeMismatch DiffEntry
        let entry = DiffEntry::type_mismatch(path.clone(), false, true);

        // Then: The entry correctly records both types
        assert_eq!(entry.path, path);
        assert_eq!(entry.diff_type, DiffType::TypeMismatch);
        assert_eq!(entry.left_is_dir, Some(false));
        assert_eq!(entry.right_is_dir, Some(true));
    }

    // ========================================
    // Edge cases
    // ========================================

    #[test]
    fn test_empty_file_vs_content() {
        // Given: An empty file on left, file with content on right
        let (left, right) = setup_test_dirs();
        fs::write(left.path().join("file.txt"), "").unwrap();
        fs::write(right.path().join("file.txt"), "content").unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: One Modified diff is found
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::Modified);
    }

    #[test]
    fn test_empty_directory_in_both() {
        // Given: An empty subdirectory in both left and right
        let (left, right) = setup_test_dirs();
        fs::create_dir(left.path().join("empty_dir")).unwrap();
        fs::create_dir(right.path().join("empty_dir")).unwrap();

        // When: Comparing the directories
        let diffs = compare_directories(left.path(), right.path()).unwrap();

        // Then: No differences are found
        assert!(diffs.is_empty());
    }
}
