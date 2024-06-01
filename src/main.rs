mod manager;

use manager::Manager;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
#[command(version)]
struct Args {
    /// Config file to load
    #[arg(short, long, value_name = "file", required = true)]
    file: String,

    /// Deploy a config
    #[arg(short, long, value_name = "config")]
    deploy: Option<String>,

    /// Display all entries
    #[arg(short, long, value_name = "boolean", action = ArgAction::SetTrue)]
    list: bool,

    /// Display full config
    #[arg(long, value_name = "boolean", action = ArgAction::SetTrue)]
    list_full: bool,

    /// Display full config
    #[arg(short = 'r', long, value_name = "boolean", action = ArgAction::SetTrue)]
    dry_run: bool,
}


fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let text = std::fs::read_to_string(args.file)?;
    let manager = Manager::new(text.as_str(), args.dry_run)?;

    if let Some(config) = args.deploy {
        manager.deploy_config(&config);
    }
    if args.list {
        manager.list_entries();
    }
    if args.list_full {
        manager.list_full_config();
    }

    Ok(())
}
