use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mountify", version, about = "Globally mounted modules via OverlayFS")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run installation checks (replaces customize.sh)
    Install {
        #[arg(short, long)]
        modpath: Option<String>,
    },

    /// Run the main mount logic (replaces post-fs-data.sh / metamount.sh)
    Mount {
        /// Path to the module directory
        #[arg(short, long, default_value = "/data/adb/modules/mountify-rs")]
        moddir: String,

        /// If set, run as metamodule (metamount.sh mode)
        #[arg(short, long)]
        metamodule: bool,
    },

    /// Run service operations (replaces service.sh)
    Service,

    /// Handle boot completed tasks (replaces boot-completed.sh)
    BootCompleted,

    /// Handle uninstall cleanup (replaces uninstall.sh)
    Uninstall,

    /// Show mount status (replaces action.sh)
    Status,

    /// List currently mounted modules
    Mounted,

    /// Generate whiteout module (replaces whiteout_gen.sh)
    WhiteoutGen {
        /// Optional whiteout list file
        #[arg(default_value = "/data/adb/mountify/whiteouts.txt")]
        list: String,

        /// Output directory
        #[arg(short, long, default_value = "/data/adb/modules_update/mountify_whiteouts")]
        output: String,
    },

    /// Metainstall hook
    Metainstall {
        /// Module ID
        #[arg(short, long)]
        modid: Option<String>,

        /// Module path
        #[arg(short, long)]
        modpath: Option<String>,
    },
}
