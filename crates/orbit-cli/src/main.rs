use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use orbit_core::{self, OrbitValue};

#[derive(Parser)]
#[command(name = "orbit", version, about = "Orbit configuration language CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a file and validate syntax
    Parse { input: PathBuf },
    /// Print the AST for a file
    Ast { input: PathBuf },
    /// Evaluate a file and print the resulting value
    Eval {
        input: PathBuf,
        /// Output as JSON instead of YAML
        #[arg(long)]
        json: bool,
    },
    /// Format a file using the canonical Orbit style
    Format {
        input: PathBuf,
        /// Write the formatted output back to the file
        #[arg(long)]
        write: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Parse { input } => parse_file(&input),
        Commands::Ast { input } => print_ast(&input),
        Commands::Eval { input, json } => eval_file(&input, json),
        Commands::Format { input, write } => format_file(&input, write),
    }
}

fn parse_file(path: &PathBuf) -> Result<()> {
    let source = read_file(path)?;
    let report = orbit_core::parse_with_recovery(&source)?;
    if report.errors.is_empty() {
        println!("{} parsed successfully", path.display());
        Ok(())
    } else {
        for error in &report.errors {
            eprintln!("{}", error);
        }
        bail!(
            "{} parse error(s) emitted while processing {}",
            report.errors.len(),
            path.display()
        );
    }
}

fn print_ast(path: &PathBuf) -> Result<()> {
    let source = read_file(path)?;
    let ast = orbit_core::parse(&source)?;
    let json = serde_json::to_string_pretty(&ast)?;
    println!("{}", json);
    Ok(())
}

fn eval_file(path: &PathBuf, json: bool) -> Result<()> {
    let source = read_file(path)?;
    let value = orbit_core::evaluate(&source)?;
    if json {
        print_json(&value)?;
    } else {
        print_yaml(&value)?;
    }
    Ok(())
}

fn format_file(path: &PathBuf, write_back: bool) -> Result<()> {
    let source = read_file(path)?;
    let formatted = orbit_fmt::format_source(&source)?;
    if write_back {
        fs::write(path, formatted)?;
        println!("{} formatted", path.display());
    } else {
        print!("{}", formatted);
    }
    Ok(())
}

fn read_file(path: &PathBuf) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

fn print_json(value: &OrbitValue) -> Result<()> {
    let out = serde_json::to_string_pretty(value)?;
    println!("{}", out);
    Ok(())
}

fn print_yaml(value: &OrbitValue) -> Result<()> {
    let out = serde_yaml::to_string(value)?;
    println!("{}", out.trim_end());
    Ok(())
}
