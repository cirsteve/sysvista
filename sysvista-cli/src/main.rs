use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use sysvista_cli::{bundle, discovery, output, scanner};

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum OutputFormat {
    V1,
    #[default]
    V2,
}

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

        /// Output path (a directory for v2, a JSON file for v1)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output contract version
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Write the canonical v2 JSON Schema
    Schema {
        /// Output path (defaults to schema/sysvista-v2.schema.json at the repository root)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Pack a v2 directory bundle into a portable zip archive
    Bundle {
        /// Directory bundle to pack
        #[arg(long, default_value = "sysvista-output")]
        input: PathBuf,
        /// Destination zip path
        #[arg(long)]
        archive: PathBuf,
        /// Omit content-addressed source bytes
        #[arg(long)]
        no_source: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            path,
            output,
            format,
        } => {
            let root = path.canonicalize().unwrap_or_else(|e| {
                eprintln!("Error: cannot resolve path '{}': {e}", path.display());
                std::process::exit(1);
            });

            eprintln!("Scanning {}...", root.display());

            match format {
                OutputFormat::V1 => {
                    let output = output.unwrap_or_else(|| PathBuf::from("sysvista-output.json"));
                    let result = scanner::scan(&root);
                    output::writer::write_json(&result, &output).unwrap_or_else(|e| {
                        eprintln!("Error writing output: {e}");
                        std::process::exit(1);
                    });
                    eprintln!("Output written to {}", output.display());
                }
                OutputFormat::V2 => {
                    let config = discovery::Config::load(&root).unwrap_or_else(|e| {
                        eprintln!("Error loading sysvista.toml: {e}");
                        std::process::exit(1);
                    });
                    let output = output.unwrap_or_else(|| PathBuf::from("sysvista-output"));
                    let snapshot = scanner::scan_v2(&root, &config).unwrap_or_else(|e| {
                        eprintln!("Error scanning project: {e}");
                        std::process::exit(1);
                    });
                    output::v2::write_bundle(&snapshot, &output).unwrap_or_else(|e| {
                        eprintln!("Error writing output: {e}");
                        std::process::exit(1);
                    });
                    eprintln!("Output written to {}", output.display());
                }
            }
        }
        Commands::Schema { output } => {
            let output = output.unwrap_or_else(output::v2::default_schema_path);
            output::v2::write_schema(&output).unwrap_or_else(|e| {
                eprintln!("Error writing v2 schema to '{}': {e}", output.display());
                std::process::exit(1);
            });
            eprintln!("Schema written to {}", output.display());
        }
        Commands::Bundle {
            input,
            archive,
            no_source,
        } => {
            let options = bundle::ArchiveOptions {
                include_source: !no_source,
                ..Default::default()
            };
            bundle::write_archive(&input, &archive, &options).unwrap_or_else(|e| {
                eprintln!("Error writing archive: {e}");
                std::process::exit(1);
            });
            eprintln!("Archive written to {}", archive.display());
        }
    }
}
