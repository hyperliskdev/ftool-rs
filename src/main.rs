use clap::{Parser, Subcommand};
use rusty_falcon::{
    easy::client::{FalconHandle},
};
use std::path::PathBuf;
use tools::tag_hosts::tag_hosts;

mod tools;


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
        #[arg(short, long, value_name = "TAG", num_args(0..=10))]
        tag: Vec<String>,

        #[arg(short, long, value_name = "FILE")]
        /// The hosts to tag
        hosts: Option<PathBuf>,

        #[arg(short, long, value_name = "ACTION", default_value="add")]
        action: String,
    },
}
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let falcon = FalconHandle::from_env()
        .await
        .expect("Could not authenticate with CrowdStrike API");

    match &cli.command {
        Some(Commands::TagHosts { tag, hosts, action }) => {
            
            // Check if a file exists and pass the path to the tag_hosts function
            let hosts = match hosts {
                Some(path) => {
                    if path.exists() {
                        path.clone()
                    } else {
                        eprintln!("The specified file does not exist: {:?}", path);
                        return;
                    }
                }
                None => {
                    eprintln!("No hosts file provided. Please specify a file with the --hosts option.");
                    return;
                }
            };
            
            let result = tag_hosts(
                &falcon,
                tag.clone(),
                hosts,
                action.clone(),
            )
            .await
            .expect("Failed to tag hosts");

            println!("{}", result);
        }
        None => {
            eprintln!("No command provided. Use --help for more information.");
        }
    }
}
