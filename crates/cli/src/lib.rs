use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    Start,
    Seed,
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Load services from a YAML file
    Load {
        /// Path to the YAML configuration file
        #[arg(short, long)]
        file: String,
        
        /// Skip duplicate services (don't create if service with same name exists)
        #[arg(short, long, default_value = "true")]
        skip_duplicates: bool,
    },
    /// Load services from a directory containing YAML files
    LoadDir {
        /// Path to the directory containing YAML files
        #[arg(short, long)]
        dir: String,
    },
    /// Load services from default configuration locations
    LoadDefault,
}

#[derive(Subcommand)]
pub enum MetricsCommands {
    /// Calculate metrics for all services
    Calculate,
    /// Calculate metrics for a specific service
    CalculateService {
        /// Service ID
        service_id: String,
    },
    /// Calculate metrics for yesterday
    CalculateYesterday,
    /// Clean up old metrics
    Cleanup {
        /// Delete metrics older than this many days
        #[arg(default_value = "90")]
        days: u32,
    },
} 