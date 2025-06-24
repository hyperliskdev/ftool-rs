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
        #[arg(short, long, value_name = "TAG")]
        tag: String,

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
            tag_hosts(
                &falcon,
                tag.clone(),
                hosts.clone(),
                action.clone(),
            )
            .await
            .expect("Failed to tag hosts");
        }
        None => {
            eprintln!("No command provided. Use --help for more information.");
        }
    }
}
