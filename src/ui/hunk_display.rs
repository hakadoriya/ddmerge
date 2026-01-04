use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;

use crate::diff::{Hunk, HunkChoice};

/// Check if a hunk contains only whitespace differences
fn is_whitespace_only_diff(hunk: &Hunk) -> bool {
    // Combine all left and right lines, strip whitespace, and compare
    let left_stripped: String = hunk
        .left_lines
        .iter()
        .flat_map(|s| s.chars().filter(|c| !c.is_whitespace()))
        .collect();
    let right_stripped: String = hunk
        .right_lines
        .iter()
        .flat_map(|s| s.chars().filter(|c| !c.is_whitespace()))
        .collect();
    left_stripped == right_stripped
}

/// Visualize whitespace characters in a line
fn visualize_whitespace(line: &str) -> String {
    line.chars()
        .map(|c| match c {
            ' ' => '·',
            '\t' => '→',
            '\n' => '↵',
            '\r' => '␍',
            _ => c,
        })
        .collect()
}

/// Display a hunk with colored output
pub fn display_hunk(hunk: &Hunk, index: usize, total: usize, file_path: &Path) {
    let whitespace_only = is_whitespace_only_diff(hunk);

    println!();
    if whitespace_only {
        println!(
            "{} {} in {} {}",
            format!("[{}/{}]", index + 1, total).cyan().bold(),
            "Hunk".white().bold(),
            file_path.display().to_string().white(),
            "(whitespace only)".yellow()
        );
    } else {
        println!(
            "{} {} in {}",
            format!("[{}/{}]", index + 1, total).cyan().bold(),
            "Hunk".white().bold(),
            file_path.display().to_string().white()
        );
    }

    // Show hunk header
    println!(
        "  {} @@ -{},{} +{},{} @@",
        "".dimmed(),
        hunk.left_start + 1,
        hunk.left_count,
        hunk.right_start + 1,
        hunk.right_count
    );

    // Show context before
    for line in &hunk.context_before {
        let display_line = if whitespace_only {
            visualize_whitespace(line)
        } else {
            line.trim_end().to_string()
        };
        println!("  {}", format!(" {}", display_line).dimmed());
    }

    // Show left lines (what would be removed/changed)
    for line in &hunk.left_lines {
        let display_line = if whitespace_only {
            visualize_whitespace(line)
        } else {
            line.trim_end().to_string()
        };
        println!("  {}", format!("-{}", display_line).red());
    }

    // Show right lines (what would be added/changed)
    for line in &hunk.right_lines {
        let display_line = if whitespace_only {
            visualize_whitespace(line)
        } else {
            line.trim_end().to_string()
        };
        println!("  {}", format!("+{}", display_line).green());
    }

    // Show context after
    for line in &hunk.context_after {
        let display_line = if whitespace_only {
            visualize_whitespace(line)
        } else {
            line.trim_end().to_string()
        };
        println!("  {}", format!(" {}", display_line).dimmed());
    }
}

/// User choice result for hunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkUserChoice {
    Choice(HunkChoice),
    SkipFile,
    Quit,
}

/// Prompt user for hunk choice
pub fn prompt_for_hunk_choice() -> HunkUserChoice {
    println!();
    print!(
        "  Choose: {}eft (update right) / {}ight (update left) / {}kip / ",
        "(l)".red().bold(),
        "(r)".green().bold(),
        "(s)".yellow().bold()
    );
    print!(
        "skip {}ile / {}uit > ",
        "(f)".yellow().bold(),
        "(q)".magenta().bold()
    );
    io::stdout().flush().unwrap();

    loop {
        let input = read_single_char();
        match input.to_lowercase().as_str() {
            "l" => {
                println!("{}", " Using left (will update right file)".green());
                return HunkUserChoice::Choice(HunkChoice::Left);
            }
            "r" => {
                println!("{}", " Using right (will update left file)".green());
                return HunkUserChoice::Choice(HunkChoice::Right);
            }
            "s" => {
                println!("{}", " Skipped".yellow());
                return HunkUserChoice::Choice(HunkChoice::Skip);
            }
            "f" => {
                println!("{}", " Skipping file...".yellow());
                return HunkUserChoice::SkipFile;
            }
            "q" => {
                println!("{}", " Quitting...".red());
                return HunkUserChoice::Quit;
            }
            _ => {
                // Invalid input, wait for valid key
            }
        }
    }
}

fn read_single_char() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hunk(left_lines: Vec<&str>, right_lines: Vec<&str>) -> Hunk {
        Hunk {
            left_start: 0,
            left_count: left_lines.len(),
            right_start: 0,
            right_count: right_lines.len(),
            left_lines: left_lines.into_iter().map(String::from).collect(),
            right_lines: right_lines.into_iter().map(String::from).collect(),
            context_before: vec![],
            context_after: vec![],
        }
    }

    #[test]
    fn test_visualize_whitespace_space() {
        // Given: A string containing a space

        // When: Visualizing whitespace
        let result = visualize_whitespace("hello world");

        // Then: Space is replaced with visible marker
        assert_eq!(result, "hello·world");
    }

    #[test]
    fn test_visualize_whitespace_tab() {
        // Given: A string containing a tab

        // When: Visualizing whitespace
        let result = visualize_whitespace("hello\tworld");

        // Then: Tab is replaced with visible marker
        assert_eq!(result, "hello→world");
    }

    #[test]
    fn test_visualize_whitespace_newline() {
        // Given: A string containing a newline

        // When: Visualizing whitespace
        let result = visualize_whitespace("hello\n");

        // Then: Newline is replaced with visible marker
        assert_eq!(result, "hello↵");
    }

    #[test]
    fn test_visualize_whitespace_carriage_return() {
        // Given: A string containing CRLF

        // When: Visualizing whitespace
        let result = visualize_whitespace("hello\r\n");

        // Then: Both CR and LF are replaced with visible markers
        assert_eq!(result, "hello␍↵");
    }

    #[test]
    fn test_visualize_whitespace_mixed() {
        // Given: A string containing multiple whitespace types

        // When: Visualizing whitespace
        let result = visualize_whitespace(" \t\n");

        // Then: All whitespace characters are replaced
        assert_eq!(result, "·→↵");
    }

    #[test]
    fn test_visualize_whitespace_no_whitespace() {
        // Given: A string with no whitespace

        // When: Visualizing whitespace
        let result = visualize_whitespace("hello");

        // Then: String remains unchanged
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_is_whitespace_only_diff_true_spaces() {
        // Given: A hunk where only the number of spaces differs
        let hunk = create_test_hunk(vec!["hello world\n"], vec!["hello  world\n"]);

        // When: Checking if it's a whitespace-only diff

        // Then: Returns true
        assert!(is_whitespace_only_diff(&hunk));
    }

    #[test]
    fn test_is_whitespace_only_diff_true_tabs_vs_spaces() {
        // Given: A hunk where tabs and spaces are interchanged
        let hunk = create_test_hunk(vec!["\thello\n"], vec!["    hello\n"]);

        // When: Checking if it's a whitespace-only diff

        // Then: Returns true
        assert!(is_whitespace_only_diff(&hunk));
    }

    #[test]
    fn test_is_whitespace_only_diff_true_trailing_newline() {
        // Given: A hunk where only trailing newline differs
        let hunk = create_test_hunk(vec!["hello"], vec!["hello\n"]);

        // When: Checking if it's a whitespace-only diff

        // Then: Returns true
        assert!(is_whitespace_only_diff(&hunk));
    }

    #[test]
    fn test_is_whitespace_only_diff_false_different_content() {
        // Given: A hunk with different non-whitespace content
        let hunk = create_test_hunk(vec!["hello\n"], vec!["world\n"]);

        // When: Checking if it's a whitespace-only diff

        // Then: Returns false
        assert!(!is_whitespace_only_diff(&hunk));
    }

    #[test]
    fn test_is_whitespace_only_diff_false_additional_content() {
        // Given: A hunk where right has additional non-whitespace content
        let hunk = create_test_hunk(vec!["hello\n"], vec!["hello world\n"]);

        // When: Checking if it's a whitespace-only diff

        // Then: Returns false
        assert!(!is_whitespace_only_diff(&hunk));
    }

    #[test]
    fn test_is_whitespace_only_diff_empty_lines() {
        // Given: A hunk with no lines on either side
        let hunk = create_test_hunk(vec![], vec![]);

        // When: Checking if it's a whitespace-only diff

        // Then: Returns true (no non-whitespace difference)
        assert!(is_whitespace_only_diff(&hunk));
    }

    #[test]
    fn test_is_whitespace_only_diff_one_side_empty() {
        // Given: A hunk with whitespace-only content on one side
        let hunk = create_test_hunk(vec!["   \n"], vec![]);

        // When: Checking if it's a whitespace-only diff

        // Then: Returns true (left has only whitespace)
        assert!(is_whitespace_only_diff(&hunk));
    }
}
