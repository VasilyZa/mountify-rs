use clap::Parser;
use mountify::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Install { modpath } => {
            mountify::commands::install::run(modpath.clone())
        }
        Commands::Mount { moddir, metamodule } => {
            mountify::commands::mount::run(moddir, *metamodule)
        }
        Commands::Service => {
            mountify::commands::service::run()
        }
        Commands::BootCompleted => {
            mountify::commands::boot::run()
        }
        Commands::Uninstall => {
            mountify::commands::uninstall::run()
        }
        Commands::Status => {
            mountify::commands::status::run()
        }
        Commands::WhiteoutGen { list, output } => {
            mountify::commands::whiteout::run(list, output)
        }
        Commands::Metainstall { modid, modpath } => {
            mountify::commands::metainstall::run(modid.clone(), modpath.clone())
        }
        Commands::Mounted => {
            mountify::commands::mounted::run()
        }
    };

    if let Err(e) = result {
        eprintln!("mountify: error: {}", e);
        std::process::exit(1);
    }
}
