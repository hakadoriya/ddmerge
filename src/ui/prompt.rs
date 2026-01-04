use colored::Colorize;
use std::io::{self, Write};

use crate::diff::DiffType;
use crate::merge::MergeAction;

/// Result of user interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserChoice {
    /// User selected an action
    Action(MergeAction),
    /// User wants to quit
    Quit,
}

/// Prompt user for action on a diff entry
pub fn prompt_for_action(diff_type: &DiffType) -> UserChoice {
    match diff_type {
        DiffType::LeftOnly => prompt_left_only(),
        DiffType::RightOnly => prompt_right_only(),
        DiffType::Modified => prompt_modified(),
        DiffType::TypeMismatch => prompt_type_mismatch(),
    }
}

fn prompt_left_only() -> UserChoice {
    println!();
    print!(
        "  Choose: {}eep / {}elete / ",
        "(k)".cyan().bold(),
        "(d)".cyan().bold()
    );
    print!(
        "{}kip / {}uit > ",
        "(s)".yellow().bold(),
        "(q)".red().bold()
    );
    io::stdout().flush().unwrap();

    loop {
        let input = read_single_char();
        match input.to_lowercase().as_str() {
            "k" => {
                println!("{}", " Keeping file".green());
                return UserChoice::Action(MergeAction::Keep);
            }
            "d" => {
                println!("{}", " Deleting file".red());
                return UserChoice::Action(MergeAction::Delete);
            }
            "s" => {
                println!("{}", " Skipped".yellow());
                return UserChoice::Action(MergeAction::Skip);
            }
            "q" => {
                println!("{}", " Quitting...".red());
                return UserChoice::Quit;
            }
            _ => {
                // Invalid input, wait for valid key
            }
        }
    }
}

fn prompt_right_only() -> UserChoice {
    println!();
    print!(
        "  Choose: {}eep / {}elete / ",
        "(k)".cyan().bold(),
        "(d)".cyan().bold()
    );
    print!(
        "{}kip / {}uit > ",
        "(s)".yellow().bold(),
        "(q)".red().bold()
    );
    io::stdout().flush().unwrap();

    loop {
        let input = read_single_char();
        match input.to_lowercase().as_str() {
            "k" => {
                println!("{}", " Keeping file".green());
                return UserChoice::Action(MergeAction::Keep);
            }
            "d" => {
                println!("{}", " Deleting file".red());
                return UserChoice::Action(MergeAction::Delete);
            }
            "s" => {
                println!("{}", " Skipped".yellow());
                return UserChoice::Action(MergeAction::Skip);
            }
            "q" => {
                println!("{}", " Quitting...".red());
                return UserChoice::Quit;
            }
            _ => {}
        }
    }
}

fn prompt_modified() -> UserChoice {
    println!();
    print!(
        "  Choose: {}eft / {}ight / ",
        "(l)".cyan().bold(),
        "(r)".cyan().bold()
    );
    print!(
        "{}kip / {}uit > ",
        "(s)".yellow().bold(),
        "(q)".red().bold()
    );
    io::stdout().flush().unwrap();

    loop {
        let input = read_single_char();
        match input.to_lowercase().as_str() {
            "l" => {
                println!("{}", " Using left version".green());
                return UserChoice::Action(MergeAction::UseLeft);
            }
            "r" => {
                println!("{}", " Using right version".green());
                return UserChoice::Action(MergeAction::UseRight);
            }
            "s" => {
                println!("{}", " Skipped".yellow());
                return UserChoice::Action(MergeAction::Skip);
            }
            "q" => {
                println!("{}", " Quitting...".red());
                return UserChoice::Quit;
            }
            _ => {}
        }
    }
}

fn prompt_type_mismatch() -> UserChoice {
    println!();
    print!(
        "  Choose: {}eft / {}ight / ",
        "(l)".cyan().bold(),
        "(r)".cyan().bold()
    );
    print!(
        "{}kip / {}uit > ",
        "(s)".yellow().bold(),
        "(q)".red().bold()
    );
    io::stdout().flush().unwrap();

    loop {
        let input = read_single_char();
        match input.to_lowercase().as_str() {
            "l" => {
                println!("{}", " Using left version".green());
                return UserChoice::Action(MergeAction::UseLeft);
            }
            "r" => {
                println!("{}", " Using right version".green());
                return UserChoice::Action(MergeAction::UseRight);
            }
            "s" => {
                println!("{}", " Skipped".yellow());
                return UserChoice::Action(MergeAction::Skip);
            }
            "q" => {
                println!("{}", " Quitting...".red());
                return UserChoice::Quit;
            }
            _ => {}
        }
    }
}

fn read_single_char() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
