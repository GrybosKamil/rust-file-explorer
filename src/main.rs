use clap::Parser;
use owo_colors::OwoColorize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about = "A colored file explorer CLI built in Rust")]
struct Cli {
    /// Directory path to inspect (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Show hidden files and directories
    #[arg(short, long)]
    all: bool,
}

/// Converts raw byte values into human-readable strings
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Reads and formats the directory entries
fn explore_directory(dir_path: &Path, show_hidden: bool) -> std::io::Result<()> {
    let entries = fs::read_dir(dir_path)?;

    // Print header line
    println!(
        "{:<10} {:<12} {}",
        "TYPE".bold(),
        "SIZE".bold(),
        "NAME".bold()
    );
    println!("{}", "─".repeat(45).dimmed());

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        // Skip hidden files unless --all flag is passed
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();

        if is_dir {
            println!(
                "{:<10} {:<12} {}",
                "DIR".cyan().bold(),
                "-".dimmed(),
                format!("{}/", name).cyan().bold()
            );
        } else {
            let size_str = format_size(metadata.len());
            let styled_name = if metadata.permissions().readonly() {
                name.yellow().to_string()
            } else {
                name.to_string()
            };

            println!(
                "{:<10} {:<12} {}",
                "FILE".green(),
                size_str.dimmed(),
                styled_name
            );
        }
    }

    Ok(())
}

fn main() {
    let args = Cli::parse();

    if let Err(err) = explore_directory(&args.path, args.all) {
        eprintln!("{} {}", "Error:".red().bold(), err);
        std::process::exit(1);
    }
}