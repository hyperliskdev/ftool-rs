use chrono::Local;
use clap::{Parser, Subcommand};
use fern::Dispatch;
use log::{error, warn};
use rusty_falcon::easy::client::FalconHandle;
use std::{fs, path::PathBuf};
use tools::tag_hosts::tag_hosts;
extern crate dotenv;

use dotenv::dotenv;

use crate::tools::alive_hosts::alive_hosts;
mod tools;

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
     // Ensure logs folder exists
    fs::create_dir_all("logs")?;

    // Log file name based on timestamp
    let log_file_name = format!("logs/run_{}.log", Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let log_file = fern::log_file(log_file_name)?;

    Dispatch::new()
        // Global level filter
        .level(log::LevelFilter::Trace)
        // Console: show everything
        .chain(
            Dispatch::new()
                .level(log::LevelFilter::Trace)
                .format(|out, message, record| {
                    out.finish(format_args!("[{}] {}", record.level(), message))
                })
                .chain(std::io::stdout()),
        )
        // File: only Warn and above
        .chain(
            Dispatch::new()
                .level(log::LevelFilter::Warn)
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{} [{}] {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        message
                    ))
                })
                .chain(log_file),
        )
        .apply()?;

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    // Chosen command to run
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Tag hosts in Falcon
    #[command(name = "tag-hosts")]
    TagHosts {
        /// The tag to apply to the hosts
        #[arg(long, value_name = "TAG", num_args(0..=10))]
        tag: Vec<String>,

        #[arg(long, value_name = "FILE")]
        /// The hosts to tag
        hosts: Option<PathBuf>,

        #[arg(long, value_name = "ACTION", default_value = "add")]
        action: String,
    },
    // Identify if hosts are in 
    #[command(name = "alive-hosts")]
    AliveHosts {
        #[arg(long, value_name = "FILE")]
        hosts: Option<PathBuf>
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Setup logger
    setup_logger().unwrap_or_else(|e| {
        error!("Failed to set up logger: {}", e);
        std::process::exit(1);
    });

    // Parse command line arguments
    let cli = Cli::parse();

    let falcon = FalconHandle::from_env()
        .await.unwrap_or_else(|e| {
            error!("Failed to create Falcon client: {}", e);
            std::process::exit(1);
        });

    match &cli.command {
        Some(Commands::TagHosts { tag, hosts, action }) => {
            // Check if a file exists and pass the path to the tag_hosts function
            let hosts = match hosts {
                Some(path) => {
                    if path.exists() {
                        path.clone()
                    } else {
                        warn!("The specified file does not exist: {:?}", path);
                        return;
                    }
                }
                None => {
                    error!(
                        "No hosts file provided. Please specify a file with the --hosts option."
                    );
                    return;
                }
            };

            let result = tag_hosts(&falcon, tag.clone(), hosts, action.clone()).await;

            if let Err(errors) = result {
                for error in errors {
                    error!("Error tagging hosts: {:?}", error);
                }
                return;
            }

            if action == "remove" {
                for host in &result.unwrap_or_default() {
                    warn!("[CODE: {:?}] | Host with tag: {:?} removed from {:?}", host.code, tag, host.device_id);
                }
            } else {
                for host in &result.unwrap_or_default() {
                    warn!("[CODE: {:?}] | Host {:?} tagged with: {:?}", host.code, host.device_id, tag);
                }
            }

            warn!("Hosts tagged successfully");
            
        }
        Some(Commands::AliveHosts { hosts }) => {
            let hosts = match hosts {
                Some(path) => {
                    if path.exists() {
                        path.clone()
                    } else {
                        warn!("The specified file does not exist: {:?}", path);
                        return;
                    }
                }
                None => {
                    error!(
                        "No hosts file provided. Please specify a file with the --hosts option."
                    );
                    return;
                }
            };

            let result = alive_hosts(&falcon, hosts).await;

            if let Err(errors) = result {
                for error in errors {
                    error!("Error tagging hosts: {:?}", error);
                }
                return;
            }
        }
        None => {
            error!("No command provided. Use --help for more information.");
        }
    }
}
