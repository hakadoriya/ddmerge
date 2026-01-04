use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use regex::Regex;
use std::path::{Path, PathBuf};

use ddmerge::diff::file::read_text_file;
use ddmerge::diff::{compare_directories, extract_hunks, DiffType};
use ddmerge::merge::{apply_file_action, apply_hunk_merge, FileAction};
use ddmerge::ui::{display_hunk, prompt_for_hunk_choice, HunkUserChoice};

/// Interactive directory diff and merge tool
///
/// Compares two directories and allows interactive hunk-by-hunk merging.
/// Changes are applied in-place to both directories.
#[derive(Parser, Debug)]
#[command(name = "ddmerge")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Left directory to compare
    left: PathBuf,

    /// Right directory to compare
    right: PathBuf,

    /// Dry run mode (show what would be done without making changes)
    #[arg(long)]
    dry_run: bool,

    /// Skip binary files silently
    #[arg(long)]
    skip_binary: bool,

    /// Skip files in left directory matching this regex pattern
    #[arg(long)]
    exclude_regex_left: Option<String>,

    /// Skip files in right directory matching this regex pattern
    #[arg(long)]
    exclude_regex_right: Option<String>,
}

/// Check if a file is binary by reading the first few bytes
fn is_binary_file(path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut buffer = [0u8; 8192];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };

    // Check for null bytes (common indicator of binary content)
    buffer[..bytes_read].contains(&0)
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate input directories
    if !args.left.is_dir() {
        anyhow::bail!("Left path is not a directory: {}", args.left.display());
    }
    if !args.right.is_dir() {
        anyhow::bail!("Right path is not a directory: {}", args.right.display());
    }

    // Compile regex patterns
    let exclude_left_regex = args
        .exclude_regex_left
        .as_ref()
        .map(|p| Regex::new(p))
        .transpose()
        .context("Invalid regex pattern for --exclude-regex-left")?;
    let exclude_right_regex = args
        .exclude_regex_right
        .as_ref()
        .map(|p| Regex::new(p))
        .transpose()
        .context("Invalid regex pattern for --exclude-regex-right")?;

    println!("{}", "Comparing directories...".cyan());
    let diffs =
        compare_directories(&args.left, &args.right).context("Failed to compare directories")?;

    if diffs.is_empty() {
        println!("{}", "Directories are identical!".green());
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} file(s) with differences.", diffs.len()).yellow()
    );

    let mut total_hunks = 0;
    let mut left_choices = 0;
    let mut right_choices = 0;
    let mut skip_choices = 0;
    let mut quit = false;

    for diff in &diffs {
        if quit {
            break;
        }

        let path_str = diff.path.to_string_lossy();

        // Check regex exclusions based on diff type
        let should_exclude = match &diff.diff_type {
            DiffType::LeftOnly => exclude_left_regex
                .as_ref()
                .is_some_and(|re| re.is_match(&path_str)),
            DiffType::RightOnly => exclude_right_regex
                .as_ref()
                .is_some_and(|re| re.is_match(&path_str)),
            DiffType::Modified | DiffType::TypeMismatch => {
                exclude_left_regex
                    .as_ref()
                    .is_some_and(|re| re.is_match(&path_str))
                    || exclude_right_regex
                        .as_ref()
                        .is_some_and(|re| re.is_match(&path_str))
            }
        };

        if should_exclude {
            continue;
        }

        let left_path = args.left.join(&diff.path);
        let right_path = args.right.join(&diff.path);

        match &diff.diff_type {
            DiffType::LeftOnly => {
                // Check for binary file
                if args.skip_binary && is_binary_file(&left_path) {
                    continue;
                }

                println!();
                println!(
                    "{} {} (only in left)",
                    "File:".cyan().bold(),
                    diff.path.display()
                );
                print!(
                    "  Choose: {}opy to right / {}elete from left / {}kip / {}uit > ",
                    "(c)".cyan().bold(),
                    "(d)".red().bold(),
                    "(s)".yellow().bold(),
                    "(q)".magenta().bold()
                );
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                loop {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    match input.trim().to_lowercase().as_str() {
                        "c" => {
                            println!("{}", "  Copying to right...".green());
                            if !args.dry_run {
                                apply_file_action(diff, FileAction::Copy, &args.left, &args.right)?;
                            }
                            break;
                        }
                        "d" => {
                            println!("{}", "  Deleting from left...".red());
                            if !args.dry_run {
                                apply_file_action(
                                    diff,
                                    FileAction::Delete,
                                    &args.left,
                                    &args.right,
                                )?;
                            }
                            break;
                        }
                        "s" => {
                            println!("{}", "  Skipped".yellow());
                            skip_choices += 1;
                            break;
                        }
                        "q" => {
                            println!("{}", "  Quitting...".red());
                            quit = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }

            DiffType::RightOnly => {
                // Check for binary file
                if args.skip_binary && is_binary_file(&right_path) {
                    continue;
                }

                println!();
                println!(
                    "{} {} (only in right)",
                    "File:".cyan().bold(),
                    diff.path.display()
                );
                print!(
                    "  Choose: {}opy to left / {}elete from right / {}kip / {}uit > ",
                    "(c)".cyan().bold(),
                    "(d)".red().bold(),
                    "(s)".yellow().bold(),
                    "(q)".magenta().bold()
                );
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                loop {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    match input.trim().to_lowercase().as_str() {
                        "c" => {
                            println!("{}", "  Copying to left...".green());
                            if !args.dry_run {
                                apply_file_action(diff, FileAction::Copy, &args.left, &args.right)?;
                            }
                            break;
                        }
                        "d" => {
                            println!("{}", "  Deleting from right...".red());
                            if !args.dry_run {
                                apply_file_action(
                                    diff,
                                    FileAction::Delete,
                                    &args.left,
                                    &args.right,
                                )?;
                            }
                            break;
                        }
                        "s" => {
                            println!("{}", "  Skipped".yellow());
                            skip_choices += 1;
                            break;
                        }
                        "q" => {
                            println!("{}", "  Quitting...".red());
                            quit = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }

            DiffType::Modified => {
                // Read file contents
                let left_content = match read_text_file(&left_path) {
                    Ok(Some(content)) => content,
                    Ok(None) => {
                        if !args.skip_binary {
                            println!(
                                "{} {} (binary file - skipping)",
                                "File:".cyan().bold(),
                                diff.path.display()
                            );
                        }
                        continue;
                    }
                    Err(e) => {
                        println!(
                            "{} {} (error reading: {})",
                            "File:".cyan().bold(),
                            diff.path.display(),
                            e
                        );
                        continue;
                    }
                };

                let right_content = match read_text_file(&right_path) {
                    Ok(Some(content)) => content,
                    Ok(None) => {
                        if !args.skip_binary {
                            println!(
                                "{} {} (binary file - skipping)",
                                "File:".cyan().bold(),
                                diff.path.display()
                            );
                        }
                        continue;
                    }
                    Err(e) => {
                        println!(
                            "{} {} (error reading: {})",
                            "File:".cyan().bold(),
                            diff.path.display(),
                            e
                        );
                        continue;
                    }
                };

                // Extract hunks
                let hunks = extract_hunks(&left_content, &right_content, 3);

                if hunks.is_empty() {
                    continue;
                }

                println!();
                println!(
                    "{} {} ({} hunk(s))",
                    "File:".cyan().bold(),
                    diff.path.display(),
                    hunks.len()
                );

                let mut hunk_choices = Vec::new();

                for (i, hunk) in hunks.iter().enumerate() {
                    display_hunk(hunk, i, hunks.len(), &diff.path);

                    match prompt_for_hunk_choice() {
                        HunkUserChoice::Choice(choice) => {
                            match choice {
                                ddmerge::diff::HunkChoice::Left => left_choices += 1,
                                ddmerge::diff::HunkChoice::Right => right_choices += 1,
                                ddmerge::diff::HunkChoice::Skip => skip_choices += 1,
                            }
                            hunk_choices.push(choice);
                            total_hunks += 1;

                            // Apply changes immediately when left or right is chosen
                            if choice != ddmerge::diff::HunkChoice::Skip && !args.dry_run {
                                let (merged_left, merged_right) = ddmerge::diff::apply_hunk_choices(
                                    &left_content,
                                    &right_content,
                                    &hunks,
                                    &hunk_choices,
                                );
                                apply_hunk_merge(
                                    &left_path,
                                    &right_path,
                                    &merged_left,
                                    &merged_right,
                                )?;
                                println!("{}", "  ✓ Applied.".green());
                            }
                        }
                        HunkUserChoice::SkipFile => {
                            // Skip remaining hunks in this file
                            break;
                        }
                        HunkUserChoice::Quit => {
                            quit = true;
                            break;
                        }
                    }
                }

                if quit {
                    break;
                }
            }

            DiffType::TypeMismatch => {
                println!();
                println!(
                    "{} {} (type mismatch: left is {}, right is {})",
                    "File:".cyan().bold(),
                    diff.path.display(),
                    if diff.left_is_dir.unwrap_or(false) {
                        "directory"
                    } else {
                        "file"
                    },
                    if diff.right_is_dir.unwrap_or(false) {
                        "directory"
                    } else {
                        "file"
                    }
                );
                print!(
                    "  Choose: {}eft (overwrite right) / {}ight (overwrite left) / {}kip / {}uit > ",
                    "(l)".red().bold(),
                    "(r)".green().bold(),
                    "(s)".yellow().bold(),
                    "(q)".magenta().bold()
                );
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                loop {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    match input.trim().to_lowercase().as_str() {
                        "l" => {
                            println!("{}", "  Using left (updating right)...".green());
                            if !args.dry_run {
                                // Remove right, copy left to right
                                let right_path = args.right.join(&diff.path);
                                if right_path.is_dir() {
                                    std::fs::remove_dir_all(&right_path)?;
                                } else {
                                    std::fs::remove_file(&right_path)?;
                                }
                                apply_file_action(diff, FileAction::Copy, &args.left, &args.right)?;
                            }
                            left_choices += 1;
                            break;
                        }
                        "r" => {
                            println!("{}", "  Using right (updating left)...".green());
                            if !args.dry_run {
                                // Remove left, copy right to left
                                let left_path = args.left.join(&diff.path);
                                if left_path.is_dir() {
                                    std::fs::remove_dir_all(&left_path)?;
                                } else {
                                    std::fs::remove_file(&left_path)?;
                                }
                                // Need to swap for RightOnly behavior
                                let mut swapped_diff = diff.clone();
                                swapped_diff.diff_type = DiffType::RightOnly;
                                apply_file_action(
                                    &swapped_diff,
                                    FileAction::Copy,
                                    &args.left,
                                    &args.right,
                                )?;
                            }
                            right_choices += 1;
                            break;
                        }
                        "s" => {
                            println!("{}", "  Skipped".yellow());
                            skip_choices += 1;
                            break;
                        }
                        "q" => {
                            println!("{}", "  Quitting...".red());
                            quit = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Summary
    println!();
    if quit {
        println!("{}", "Merge cancelled.".yellow());
    } else if args.dry_run {
        println!("{}", "Dry run complete. No files were modified.".yellow());
    } else {
        println!("{}", "Merge complete!".green().bold());
    }

    println!();
    println!("{}", "Summary:".cyan().bold());
    if total_hunks > 0 {
        println!("  Total hunks processed: {}", total_hunks);
    }
    if left_choices > 0 {
        println!("  Left choices (updated right): {}", left_choices);
    }
    if right_choices > 0 {
        println!("  Right choices (updated left): {}", right_choices);
    }
    if skip_choices > 0 {
        println!("  Skipped: {}", skip_choices);
    }

    Ok(())
}
