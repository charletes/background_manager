use clap::{Parser, Subcommand};

mod logic;
mod os_level;

#[derive(Parser)]
#[command(name = "background_manager")]
#[command(about = "Background Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the pixel size of each monitor
    Displays,
    /// Set the specified image as background
    Change {
        /// Path to the image file
        file: String,
        /// Optional monitor number (if not specified, applies to all monitors)
        monitor: Option<usize>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Displays => logic::show_monitor_sizes(),
        Commands::Change { file, monitor } => {
            let mut args = vec![file];
            if let Some(m) = monitor {
                args.push(m.to_string());
            }
            logic::change_background(&args);
        }
    }
}
