mod output;
mod scanner;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sysvista", version, about = "System architecture visualizer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a project directory and produce a JSON architecture map
    Scan {
        /// Path to the project root
        path: PathBuf,

        /// Output JSON file path
        #[arg(short, long, default_value = "sysvista-output.json")]
        output: PathBuf,
    },
    /// Write the canonical v2 JSON Schema
    Schema {
        /// Output path (defaults to schema/sysvista-v2.schema.json at the repository root)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { path, output } => {
            let root = path.canonicalize().unwrap_or_else(|e| {
                eprintln!("Error: cannot resolve path '{}': {e}", path.display());
                std::process::exit(1);
            });

            eprintln!("Scanning {}...", root.display());

            let result = scanner::scan(&root);

            eprintln!(
                "Found {} components, {} edges across {} languages ({} files scanned in {}ms)",
                result.components.len(),
                result.edges.len(),
                result.detected_languages.len(),
                result.scan_stats.files_scanned,
                result.scan_stats.scan_duration_ms,
            );

            output::writer::write_json(&result, &output).unwrap_or_else(|e| {
                eprintln!("Error writing output: {e}");
                std::process::exit(1);
            });

            eprintln!("Output written to {}", output.display());
        }
        Commands::Schema { output } => {
            let output = output.unwrap_or_else(output::v2::default_schema_path);
            output::v2::write_schema(&output).unwrap_or_else(|e| {
                eprintln!("Error writing v2 schema to '{}': {e}", output.display());
                std::process::exit(1);
            });
            eprintln!("Schema written to {}", output.display());
        }
    }
}
