use anyhow::Result;
use std::fs;
use std::path::Path;

/// Compare two files and return whether they are identical
pub fn compare_files(left: &Path, right: &Path) -> Result<bool> {
    let left_content = fs::read(left)?;
    let right_content = fs::read(right)?;
    Ok(left_content == right_content)
}

/// Check if a file appears to be binary
pub fn is_binary(path: &Path) -> Result<bool> {
    let content = fs::read(path)?;
    // Check for null bytes in the first 8KB
    let check_len = content.len().min(8192);
    Ok(content[..check_len].contains(&0))
}

/// Get file content as string if it's a text file
pub fn read_text_file(path: &Path) -> Result<Option<String>> {
    let content = fs::read(path)?;
    // Check for null bytes
    let check_len = content.len().min(8192);
    if content[..check_len].contains(&0) {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&content).into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    // ========================================
    // compare_files tests
    // ========================================

    #[test]
    fn test_compare_files_identical() {
        // Given: Two files with identical content
        let dir = create_temp_dir();
        let left = dir.path().join("left.txt");
        let right = dir.path().join("right.txt");
        fs::write(&left, "same content").unwrap();
        fs::write(&right, "same content").unwrap();

        // When: Comparing the files
        let result = compare_files(&left, &right).unwrap();

        // Then: They are reported as identical
        assert!(result);
    }

    #[test]
    fn test_compare_files_different() {
        // Given: Two files with different content
        let dir = create_temp_dir();
        let left = dir.path().join("left.txt");
        let right = dir.path().join("right.txt");
        fs::write(&left, "left content").unwrap();
        fs::write(&right, "right content").unwrap();

        // When: Comparing the files
        let result = compare_files(&left, &right).unwrap();

        // Then: They are reported as different
        assert!(!result);
    }

    #[test]
    fn test_compare_files_empty() {
        // Given: Two empty files
        let dir = create_temp_dir();
        let left = dir.path().join("left.txt");
        let right = dir.path().join("right.txt");
        fs::write(&left, "").unwrap();
        fs::write(&right, "").unwrap();

        // When: Comparing the files
        let result = compare_files(&left, &right).unwrap();

        // Then: They are reported as identical
        assert!(result);
    }

    #[test]
    fn test_compare_files_one_empty() {
        // Given: One empty file and one with content
        let dir = create_temp_dir();
        let left = dir.path().join("left.txt");
        let right = dir.path().join("right.txt");
        fs::write(&left, "").unwrap();
        fs::write(&right, "content").unwrap();

        // When: Comparing the files
        let result = compare_files(&left, &right).unwrap();

        // Then: They are reported as different
        assert!(!result);
    }

    #[test]
    fn test_compare_files_binary() {
        // Given: Two identical binary files
        let dir = create_temp_dir();
        let left = dir.path().join("left.bin");
        let right = dir.path().join("right.bin");
        let binary_content = vec![0x00, 0x01, 0x02, 0xFF];
        fs::write(&left, &binary_content).unwrap();
        fs::write(&right, &binary_content).unwrap();

        // When: Comparing the files
        let result = compare_files(&left, &right).unwrap();

        // Then: They are reported as identical
        assert!(result);
    }

    #[test]
    fn test_compare_files_nonexistent() {
        // Given: A path to a file that doesn't exist
        let dir = create_temp_dir();
        let left = dir.path().join("nonexistent.txt");
        let right = dir.path().join("right.txt");
        fs::write(&right, "content").unwrap();

        // When: Comparing with a nonexistent file
        let result = compare_files(&left, &right);

        // Then: An error is returned
        assert!(result.is_err());
    }

    // ========================================
    // is_binary tests
    // ========================================

    #[test]
    fn test_is_binary_text_file() {
        // Given: A text file with no null bytes
        let dir = create_temp_dir();
        let path = dir.path().join("text.txt");
        fs::write(&path, "Hello, world!\nThis is text.").unwrap();

        // When: Checking if it's binary
        let result = is_binary(&path).unwrap();

        // Then: It is not detected as binary
        assert!(!result);
    }

    #[test]
    fn test_is_binary_with_null_byte() {
        // Given: A file containing a null byte
        let dir = create_temp_dir();
        let path = dir.path().join("binary.bin");
        let content = b"Hello\x00World";
        fs::write(&path, content).unwrap();

        // When: Checking if it's binary
        let result = is_binary(&path).unwrap();

        // Then: It is detected as binary
        assert!(result);
    }

    #[test]
    fn test_is_binary_empty_file() {
        // Given: An empty file
        let dir = create_temp_dir();
        let path = dir.path().join("empty.txt");
        fs::write(&path, "").unwrap();

        // When: Checking if it's binary
        let result = is_binary(&path).unwrap();

        // Then: It is not detected as binary
        assert!(!result);
    }

    #[test]
    fn test_is_binary_null_at_start() {
        // Given: A file with null byte at the beginning
        let dir = create_temp_dir();
        let path = dir.path().join("binary.bin");
        let content = b"\x00Hello World";
        fs::write(&path, content).unwrap();

        // When: Checking if it's binary
        let result = is_binary(&path).unwrap();

        // Then: It is detected as binary
        assert!(result);
    }

    #[test]
    fn test_is_binary_high_bytes() {
        // Given: A file with high byte values but no null bytes
        let dir = create_temp_dir();
        let path = dir.path().join("high.bin");
        let content = vec![0x80, 0xFF, 0xFE, 0x7F];
        fs::write(&path, content).unwrap();

        // When: Checking if it's binary
        let result = is_binary(&path).unwrap();

        // Then: It is not detected as binary (no null bytes)
        assert!(!result);
    }

    #[test]
    fn test_is_binary_utf8() {
        // Given: A UTF-8 encoded file with multi-byte characters
        let dir = create_temp_dir();
        let path = dir.path().join("utf8.txt");
        fs::write(&path, "日本語テスト 🎉").unwrap();

        // When: Checking if it's binary
        let result = is_binary(&path).unwrap();

        // Then: It is not detected as binary
        assert!(!result);
    }

    // ========================================
    // read_text_file tests
    // ========================================

    #[test]
    fn test_read_text_file_success() {
        // Given: A text file with content
        let dir = create_temp_dir();
        let path = dir.path().join("text.txt");
        fs::write(&path, "Hello, world!").unwrap();

        // When: Reading the file
        let result = read_text_file(&path).unwrap();

        // Then: The content is returned
        assert_eq!(result, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_read_text_file_binary_returns_none() {
        // Given: A binary file with null bytes
        let dir = create_temp_dir();
        let path = dir.path().join("binary.bin");
        let content = b"Hello\x00World";
        fs::write(&path, content).unwrap();

        // When: Reading the file
        let result = read_text_file(&path).unwrap();

        // Then: None is returned
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_text_file_empty() {
        // Given: An empty file
        let dir = create_temp_dir();
        let path = dir.path().join("empty.txt");
        fs::write(&path, "").unwrap();

        // When: Reading the file
        let result = read_text_file(&path).unwrap();

        // Then: Empty string is returned
        assert_eq!(result, Some("".to_string()));
    }

    #[test]
    fn test_read_text_file_utf8() {
        // Given: A file with UTF-8 multi-byte characters
        let dir = create_temp_dir();
        let path = dir.path().join("utf8.txt");
        fs::write(&path, "日本語テスト").unwrap();

        // When: Reading the file
        let result = read_text_file(&path).unwrap();

        // Then: The UTF-8 content is correctly read
        assert_eq!(result, Some("日本語テスト".to_string()));
    }

    #[test]
    fn test_read_text_file_with_newlines() {
        // Given: A text file with multiple lines
        let dir = create_temp_dir();
        let path = dir.path().join("multiline.txt");
        fs::write(&path, "line1\nline2\nline3").unwrap();

        // When: Reading the file
        let result = read_text_file(&path).unwrap();

        // Then: All lines are returned with newlines preserved
        assert_eq!(result, Some("line1\nline2\nline3".to_string()));
    }

    #[test]
    fn test_read_text_file_nonexistent() {
        // Given: A path to a file that doesn't exist
        let dir = create_temp_dir();
        let path = dir.path().join("nonexistent.txt");

        // When: Attempting to read the file
        let result = read_text_file(&path);

        // Then: An error is returned
        assert!(result.is_err());
    }
}
