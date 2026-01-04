use similar::TextDiff;

/// A single hunk (contiguous block of changes)
#[derive(Debug, Clone)]
pub struct Hunk {
    /// Starting line in left file (0-indexed)
    pub left_start: usize,
    /// Number of lines from left file
    pub left_count: usize,
    /// Starting line in right file (0-indexed)
    pub right_start: usize,
    /// Number of lines from right file
    pub right_count: usize,
    /// Lines from left file (deleted/modified)
    pub left_lines: Vec<String>,
    /// Lines from right file (inserted/modified)
    pub right_lines: Vec<String>,
    /// Context lines before the change
    pub context_before: Vec<String>,
    /// Context lines after the change
    pub context_after: Vec<String>,
}

/// Choice for a hunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkChoice {
    /// Use left version (update right file)
    Left,
    /// Use right version (update left file)
    Right,
    /// Skip this hunk (leave both files unchanged for this hunk)
    Skip,
}

/// Format a line with correct newline handling
/// Only the last line may not have a newline, depending on original content
fn format_line_with_newline(
    line: &str,
    index: usize,
    total_lines: usize,
    content_ends_with_newline: bool,
) -> String {
    let is_last_line = index == total_lines - 1;
    if is_last_line && !content_ends_with_newline {
        line.to_string() // No newline for last line if original didn't have one
    } else {
        format!("{}\n", line)
    }
}

/// Extract hunks from two text contents
pub fn extract_hunks(left_content: &str, right_content: &str, context_lines: usize) -> Vec<Hunk> {
    let left_lines_vec: Vec<&str> = left_content.lines().collect();
    let right_lines_vec: Vec<&str> = right_content.lines().collect();
    let left_ends_with_newline = left_content.ends_with('\n');
    let right_ends_with_newline = right_content.ends_with('\n');
    let diff = TextDiff::from_lines(left_content, right_content);
    let mut hunks = Vec::new();

    // Process each operation individually to match apply_hunk_choices
    for op in diff.ops() {
        match op {
            similar::DiffOp::Equal { .. } => {
                // Skip equal sections, they don't create hunks
            }
            similar::DiffOp::Delete {
                old_index,
                old_len,
                new_index,
            } => {
                let mut left_lines = Vec::new();
                for i in *old_index..(*old_index + *old_len) {
                    if i < left_lines_vec.len() {
                        left_lines.push(format_line_with_newline(
                            left_lines_vec[i],
                            i,
                            left_lines_vec.len(),
                            left_ends_with_newline,
                        ));
                    }
                }

                // Get context lines
                let context_before: Vec<String> = (old_index.saturating_sub(context_lines)
                    ..*old_index)
                    .filter_map(|i| {
                        left_lines_vec.get(i).map(|s| {
                            format_line_with_newline(
                                s,
                                i,
                                left_lines_vec.len(),
                                left_ends_with_newline,
                            )
                        })
                    })
                    .collect();
                let context_after: Vec<String> = (*old_index + *old_len
                    ..(*old_index + *old_len + context_lines).min(left_lines_vec.len()))
                    .filter_map(|i| {
                        left_lines_vec.get(i).map(|s| {
                            format_line_with_newline(
                                s,
                                i,
                                left_lines_vec.len(),
                                left_ends_with_newline,
                            )
                        })
                    })
                    .collect();

                hunks.push(Hunk {
                    left_start: *old_index,
                    left_count: *old_len,
                    right_start: *new_index,
                    right_count: 0,
                    left_lines,
                    right_lines: Vec::new(),
                    context_before,
                    context_after,
                });
            }
            similar::DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => {
                let mut right_lines = Vec::new();
                for i in *new_index..(*new_index + *new_len) {
                    if i < right_lines_vec.len() {
                        right_lines.push(format_line_with_newline(
                            right_lines_vec[i],
                            i,
                            right_lines_vec.len(),
                            right_ends_with_newline,
                        ));
                    }
                }

                // Get context lines from left (since insert happens at old_index position)
                let context_before: Vec<String> = (old_index.saturating_sub(context_lines)
                    ..*old_index)
                    .filter_map(|i| {
                        left_lines_vec.get(i).map(|s| {
                            format_line_with_newline(
                                s,
                                i,
                                left_lines_vec.len(),
                                left_ends_with_newline,
                            )
                        })
                    })
                    .collect();
                let context_after: Vec<String> = (*old_index
                    ..(*old_index + context_lines).min(left_lines_vec.len()))
                    .filter_map(|i| {
                        left_lines_vec.get(i).map(|s| {
                            format_line_with_newline(
                                s,
                                i,
                                left_lines_vec.len(),
                                left_ends_with_newline,
                            )
                        })
                    })
                    .collect();

                hunks.push(Hunk {
                    left_start: *old_index,
                    left_count: 0,
                    right_start: *new_index,
                    right_count: *new_len,
                    left_lines: Vec::new(),
                    right_lines,
                    context_before,
                    context_after,
                });
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let mut left_lines = Vec::new();
                for i in *old_index..(*old_index + *old_len) {
                    if i < left_lines_vec.len() {
                        left_lines.push(format_line_with_newline(
                            left_lines_vec[i],
                            i,
                            left_lines_vec.len(),
                            left_ends_with_newline,
                        ));
                    }
                }

                let mut right_lines = Vec::new();
                for i in *new_index..(*new_index + *new_len) {
                    if i < right_lines_vec.len() {
                        right_lines.push(format_line_with_newline(
                            right_lines_vec[i],
                            i,
                            right_lines_vec.len(),
                            right_ends_with_newline,
                        ));
                    }
                }

                // Get context lines
                let context_before: Vec<String> = (old_index.saturating_sub(context_lines)
                    ..*old_index)
                    .filter_map(|i| {
                        left_lines_vec.get(i).map(|s| {
                            format_line_with_newline(
                                s,
                                i,
                                left_lines_vec.len(),
                                left_ends_with_newline,
                            )
                        })
                    })
                    .collect();
                let context_after: Vec<String> = (*old_index + *old_len
                    ..(*old_index + *old_len + context_lines).min(left_lines_vec.len()))
                    .filter_map(|i| {
                        left_lines_vec.get(i).map(|s| {
                            format_line_with_newline(
                                s,
                                i,
                                left_lines_vec.len(),
                                left_ends_with_newline,
                            )
                        })
                    })
                    .collect();

                hunks.push(Hunk {
                    left_start: *old_index,
                    left_count: *old_len,
                    right_start: *new_index,
                    right_count: *new_len,
                    left_lines,
                    right_lines,
                    context_before,
                    context_after,
                });
            }
        }
    }

    hunks
}

/// Apply hunk choices to create merged content
/// Returns (new_left_content, new_right_content)
/// - Left choice: both files get left content
/// - Right choice: both files get right content
/// - Skip choice: left file keeps left content, right file keeps right content
pub fn apply_hunk_choices(
    left_content: &str,
    right_content: &str,
    _hunks: &[Hunk],
    choices: &[HunkChoice],
) -> (String, String) {
    let left_lines: Vec<&str> = left_content.lines().collect();
    let right_lines: Vec<&str> = right_content.lines().collect();
    let mut merged_left_lines: Vec<String> = Vec::new();
    let mut merged_right_lines: Vec<String> = Vec::new();

    // Build the merged content based on choices
    let diff = TextDiff::from_lines(left_content, right_content);
    let mut hunk_idx = 0;

    // Process all operations, not just grouped ones
    for op in diff.ops() {
        match op {
            similar::DiffOp::Equal { old_index, len, .. } => {
                // Copy equal lines (they're the same in both)
                for i in *old_index..(*old_index + *len) {
                    if i < left_lines.len() {
                        merged_left_lines.push(left_lines[i].to_string());
                        merged_right_lines.push(left_lines[i].to_string());
                    }
                }
            }
            similar::DiffOp::Delete {
                old_index, old_len, ..
            } => {
                // Lines only in left (deleted from left's perspective)
                let choice = choices.get(hunk_idx).copied().unwrap_or(HunkChoice::Skip);
                match choice {
                    HunkChoice::Left => {
                        // Keep left content in both files
                        for i in *old_index..(*old_index + *old_len) {
                            if i < left_lines.len() {
                                merged_left_lines.push(left_lines[i].to_string());
                                merged_right_lines.push(left_lines[i].to_string());
                            }
                        }
                    }
                    HunkChoice::Skip => {
                        // Left keeps left content, right doesn't have these lines
                        for i in *old_index..(*old_index + *old_len) {
                            if i < left_lines.len() {
                                merged_left_lines.push(left_lines[i].to_string());
                            }
                        }
                        // Right file: don't include (original right doesn't have these)
                    }
                    HunkChoice::Right => {
                        // Don't include left content in either (it's deleted)
                    }
                }
                hunk_idx += 1;
            }
            similar::DiffOp::Insert {
                new_index, new_len, ..
            } => {
                // Lines only in right (inserted from left's perspective)
                let choice = choices.get(hunk_idx).copied().unwrap_or(HunkChoice::Skip);
                match choice {
                    HunkChoice::Right => {
                        // Include right content in both files
                        for i in *new_index..(*new_index + *new_len) {
                            if i < right_lines.len() {
                                merged_left_lines.push(right_lines[i].to_string());
                                merged_right_lines.push(right_lines[i].to_string());
                            }
                        }
                    }
                    HunkChoice::Skip => {
                        // Left doesn't have these lines, right keeps them
                        // Left file: don't include (original left doesn't have these)
                        for i in *new_index..(*new_index + *new_len) {
                            if i < right_lines.len() {
                                merged_right_lines.push(right_lines[i].to_string());
                            }
                        }
                    }
                    HunkChoice::Left => {
                        // Don't include right content in either (not inserted)
                    }
                }
                hunk_idx += 1;
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                // Lines changed between left and right
                let choice = choices.get(hunk_idx).copied().unwrap_or(HunkChoice::Skip);
                match choice {
                    HunkChoice::Left => {
                        // Use left content in both files
                        for i in *old_index..(*old_index + *old_len) {
                            if i < left_lines.len() {
                                merged_left_lines.push(left_lines[i].to_string());
                                merged_right_lines.push(left_lines[i].to_string());
                            }
                        }
                    }
                    HunkChoice::Skip => {
                        // Each file keeps its own content
                        for i in *old_index..(*old_index + *old_len) {
                            if i < left_lines.len() {
                                merged_left_lines.push(left_lines[i].to_string());
                            }
                        }
                        for i in *new_index..(*new_index + *new_len) {
                            if i < right_lines.len() {
                                merged_right_lines.push(right_lines[i].to_string());
                            }
                        }
                    }
                    HunkChoice::Right => {
                        // Use right content in both files
                        for i in *new_index..(*new_index + *new_len) {
                            if i < right_lines.len() {
                                merged_left_lines.push(right_lines[i].to_string());
                                merged_right_lines.push(right_lines[i].to_string());
                            }
                        }
                    }
                }
                hunk_idx += 1;
            }
        }
    }

    // Determine trailing newline behavior based on choices
    let left_has_newline = left_content.ends_with('\n');
    let right_has_newline = right_content.ends_with('\n');

    // Find the last non-skip choice to determine trailing newline behavior
    let last_choice = choices
        .iter()
        .rev()
        .find(|c| **c != HunkChoice::Skip)
        .copied();

    let (left_trailing, right_trailing) = match last_choice {
        Some(HunkChoice::Left) => {
            // Left wins: both files should use left's trailing newline
            (left_has_newline, left_has_newline)
        }
        Some(HunkChoice::Right) => {
            // Right wins: both files should use right's trailing newline
            (right_has_newline, right_has_newline)
        }
        _ => {
            // All skipped or no choices: preserve original behavior
            (left_has_newline, right_has_newline)
        }
    };

    // Join lines with newline and apply trailing newline
    let mut merged_left = merged_left_lines.join("\n");
    let mut merged_right = merged_right_lines.join("\n");
    if left_trailing && !merged_left.is_empty() {
        merged_left.push('\n');
    }
    if right_trailing && !merged_right.is_empty() {
        merged_right.push('\n');
    }

    (merged_left, merged_right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hunks_simple() {
        // Given: Two files with one line different
        let left = "line1\nline2\nline3\n";
        let right = "line1\nmodified\nline3\n";

        // When: Extracting hunks with context
        let hunks = extract_hunks(left, right, 1);

        // Then: One hunk is found with the changed lines
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].left_lines, vec!["line2\n"]);
        assert_eq!(hunks[0].right_lines, vec!["modified\n"]);
    }

    #[test]
    fn test_extract_hunks_multiple() {
        // Given: Two files with multiple lines different
        let left = "a\nb\nc\nd\ne\n";
        let right = "a\nB\nc\nD\ne\n";

        // When: Extracting hunks without context
        let hunks = extract_hunks(left, right, 0);

        // Then: At least one hunk is found
        assert!(!hunks.is_empty());
    }

    #[test]
    fn test_apply_hunk_choices_left() {
        // Given: Two files with different content and Left choice
        let left = "line1\nold\nline3\n";
        let right = "line1\nnew\nline3\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Left];

        // When: Applying hunk choices
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Both files have left's content
        assert_eq!(merged_left, "line1\nold\nline3\n");
        assert_eq!(merged_right, "line1\nold\nline3\n");
    }

    #[test]
    fn test_apply_hunk_choices_right() {
        // Given: Two files with different content and Right choice
        let left = "line1\nold\nline3\n";
        let right = "line1\nnew\nline3\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Right];

        // When: Applying hunk choices
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Both files have right's content
        assert_eq!(merged_left, "line1\nnew\nline3\n");
        assert_eq!(merged_right, "line1\nnew\nline3\n");
    }

    #[test]
    fn test_apply_hunk_choices_skip() {
        // Given: Two files with different content and Skip choice
        let left = "line1\nold\nline3\n";
        let right = "line1\nnew\nline3\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Skip];

        // When: Applying hunk choices
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Each file keeps its original content
        assert_eq!(merged_left, left);
        assert_eq!(merged_right, right);
    }

    #[test]
    fn test_trailing_newline_left_choice() {
        // Given: Left has no trailing newline, right has trailing newline
        let left = "hello";
        let right = "hello\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Left];

        // When: Applying hunk choices with Left
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Both files have no trailing newline (left's behavior)
        assert!(!merged_left.ends_with('\n'));
        assert!(!merged_right.ends_with('\n'));
    }

    #[test]
    fn test_trailing_newline_right_choice() {
        // Given: Left has no trailing newline, right has trailing newline
        let left = "hello";
        let right = "hello\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Right];

        // When: Applying hunk choices with Right
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Both files have trailing newline (right's behavior)
        assert!(merged_left.ends_with('\n'));
        assert!(merged_right.ends_with('\n'));
    }

    #[test]
    fn test_trailing_newline_skip_preserves_original() {
        // Given: Left has no trailing newline, right has trailing newline
        let left = "hello";
        let right = "hello\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Skip];

        // When: Applying hunk choices with Skip
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Each file preserves its original trailing newline behavior
        assert!(!merged_left.ends_with('\n'));
        assert!(merged_right.ends_with('\n'));
    }

    #[test]
    fn test_extract_hunks_delete_operation() {
        // Given: Right file has a line deleted compared to left
        let left = "line1\nline2\nline3\n";
        let right = "line1\nline3\n";

        // When: Extracting hunks
        let hunks = extract_hunks(left, right, 0);

        // Then: One hunk is found with delete operation (left has line, right is empty)
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].left_lines, vec!["line2\n"]);
        assert!(hunks[0].right_lines.is_empty());
    }

    #[test]
    fn test_extract_hunks_insert_operation() {
        // Given: Right file has a line inserted compared to left
        let left = "line1\nline3\n";
        let right = "line1\nline2\nline3\n";

        // When: Extracting hunks
        let hunks = extract_hunks(left, right, 0);

        // Then: One hunk is found with insert operation (left is empty, right has line)
        assert_eq!(hunks.len(), 1);
        assert!(hunks[0].left_lines.is_empty());
        assert_eq!(hunks[0].right_lines, vec!["line2\n"]);
    }

    #[test]
    fn test_apply_hunk_choices_delete_left() {
        // Given: Right has a line deleted, and Left choice is made
        let left = "line1\nline2\nline3\n";
        let right = "line1\nline3\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Left];

        // When: Applying hunk choices
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Both files keep the line (left's version with the line)
        assert_eq!(merged_left, "line1\nline2\nline3\n");
        assert_eq!(merged_right, "line1\nline2\nline3\n");
    }

    #[test]
    fn test_apply_hunk_choices_delete_right() {
        // Given: Right has a line deleted, and Right choice is made
        let left = "line1\nline2\nline3\n";
        let right = "line1\nline3\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Right];

        // When: Applying hunk choices
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Both files have the line deleted (right's version without the line)
        assert_eq!(merged_left, "line1\nline3\n");
        assert_eq!(merged_right, "line1\nline3\n");
    }

    #[test]
    fn test_apply_hunk_choices_insert_left() {
        // Given: Right has a line inserted, and Left choice is made
        let left = "line1\nline3\n";
        let right = "line1\nline2\nline3\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Left];

        // When: Applying hunk choices
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Both files don't have the inserted line (left's version)
        assert_eq!(merged_left, "line1\nline3\n");
        assert_eq!(merged_right, "line1\nline3\n");
    }

    #[test]
    fn test_apply_hunk_choices_insert_right() {
        // Given: Right has a line inserted, and Right choice is made
        let left = "line1\nline3\n";
        let right = "line1\nline2\nline3\n";
        let hunks = extract_hunks(left, right, 0);
        let choices = vec![HunkChoice::Right];

        // When: Applying hunk choices
        let (merged_left, merged_right) = apply_hunk_choices(left, right, &hunks, &choices);

        // Then: Both files have the inserted line (right's version)
        assert_eq!(merged_left, "line1\nline2\nline3\n");
        assert_eq!(merged_right, "line1\nline2\nline3\n");
    }

    #[test]
    fn test_no_difference() {
        // Given: Two identical files
        let content = "line1\nline2\nline3\n";

        // When: Extracting hunks
        let hunks = extract_hunks(content, content, 0);

        // Then: No hunks are found
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_empty_files() {
        // Given: Two empty files
        let left = "";
        let right = "";

        // When: Extracting hunks
        let hunks = extract_hunks(left, right, 0);

        // Then: No hunks are found
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_format_line_with_newline_last_line_no_newline() {
        // Given: Last line of a file that originally had no trailing newline

        // When: Formatting the line
        let result = format_line_with_newline("hello", 0, 1, false);

        // Then: No newline is added
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_line_with_newline_last_line_with_newline() {
        // Given: Last line of a file that originally had trailing newline

        // When: Formatting the line
        let result = format_line_with_newline("hello", 0, 1, true);

        // Then: Newline is added
        assert_eq!(result, "hello\n");
    }

    #[test]
    fn test_format_line_with_newline_not_last_line() {
        // Given: A line that is not the last line

        // When: Formatting the line
        let result = format_line_with_newline("hello", 0, 2, false);

        // Then: Newline is always added for non-last lines
        assert_eq!(result, "hello\n");
    }
}
