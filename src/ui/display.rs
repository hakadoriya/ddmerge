use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

use crate::diff::file::read_text_file;
use crate::diff::{DiffEntry, DiffType};

/// Display a diff entry with colored output
pub fn display_diff(
    entry: &DiffEntry,
    index: usize,
    total: usize,
    left_root: &Path,
    right_root: &Path,
) {
    println!();
    println!(
        "{} {}",
        format!("[{}/{}]", index + 1, total).cyan().bold(),
        format!("File: {}", entry.path.display()).white().bold()
    );

    match &entry.diff_type {
        DiffType::LeftOnly => {
            let is_dir = entry.left_is_dir.unwrap_or(false);
            let type_str = if is_dir { "directory" } else { "file" };
            println!("  {} (only in left)", type_str.yellow());

            if !is_dir {
                show_file_info(&left_root.join(&entry.path), "Left");
            }
        }
        DiffType::RightOnly => {
            let is_dir = entry.right_is_dir.unwrap_or(false);
            let type_str = if is_dir { "directory" } else { "file" };
            println!("  {} (only in right)", type_str.yellow());

            if !is_dir {
                show_file_info(&right_root.join(&entry.path), "Right");
            }
        }
        DiffType::Modified => {
            let left_path = left_root.join(&entry.path);
            let right_path = right_root.join(&entry.path);

            show_file_info(&left_path, "Left");
            show_file_info(&right_path, "Right");

            // Show text diff if possible
            show_text_diff(&left_path, &right_path);
        }
        DiffType::TypeMismatch => {
            let left_type = if entry.left_is_dir.unwrap_or(false) {
                "directory"
            } else {
                "file"
            };
            let right_type = if entry.right_is_dir.unwrap_or(false) {
                "directory"
            } else {
                "file"
            };
            println!(
                "  {} Left is {}, Right is {}",
                "Type mismatch:".red().bold(),
                left_type.yellow(),
                right_type.yellow()
            );
        }
    }
}

fn show_file_info(path: &Path, side: &str) {
    if let Ok(metadata) = fs::metadata(path) {
        let size = metadata.len();
        let size_str = format_size(size);

        if let Ok(modified) = metadata.modified() {
            let datetime: chrono::DateTime<chrono::Local> = modified.into();
            println!(
                "  {}: modified {}, {}",
                side.cyan(),
                datetime.format("%Y-%m-%d %H:%M"),
                size_str
            );
        } else {
            println!("  {}: {}", side.cyan(), size_str);
        }
    }
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{}B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1}KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn show_text_diff(left_path: &Path, right_path: &Path) {
    let left_content = match read_text_file(left_path) {
        Ok(Some(content)) => content,
        Ok(None) => {
            println!("  {}", "(binary file)".dimmed());
            return;
        }
        Err(_) => return,
    };

    let right_content = match read_text_file(right_path) {
        Ok(Some(content)) => content,
        Ok(None) => {
            println!("  {}", "(binary file)".dimmed());
            return;
        }
        Err(_) => return,
    };

    println!();
    println!(
        "  {} {}",
        "---".red(),
        format!(
            "left/{}",
            left_path.file_name().unwrap_or_default().to_string_lossy()
        )
        .red()
    );
    println!(
        "  {} {}",
        "+++".green(),
        format!(
            "right/{}",
            right_path.file_name().unwrap_or_default().to_string_lossy()
        )
        .green()
    );

    let diff = TextDiff::from_lines(&left_content, &right_content);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("  {}", "...".dimmed());
        }

        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, style): (&str, fn(&str) -> colored::ColoredString) = match change.tag() {
                    ChangeTag::Delete => ("-", |s: &str| s.red()),
                    ChangeTag::Insert => ("+", |s: &str| s.green()),
                    ChangeTag::Equal => (" ", |s: &str| s.normal()),
                };

                let line = format!("  {}{}", sign, change.value().trim_end());
                println!("{}", style(&line));
            }
        }
    }
}
