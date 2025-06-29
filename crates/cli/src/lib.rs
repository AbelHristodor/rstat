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
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },
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