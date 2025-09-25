use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cave", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command : Command,
}

#[derive(Subcommand)]
pub enum Command {
    ///Define the default version
    Use {
        ///Code aster version : stable, testing or under this format : 1x.x.xx
        version : String,
    },
    ///Define the directory version
    Pin {
        ///Code aster version : stable, testing or under this format : 1x.x.xx 
        version : String,
    },
    ///Run code_aster 
    #[command(override_usage = "cave run -- [ARGS]")]
    Run {
        ///Optional args followed by export file 
        #[arg(trailing_var_arg = true)]
        #[arg(value_name = "ARGS")]
        args: Vec<String>,
    },
    ///List downloaded images 
    List {
        ///Optionnal Expression to match, ex : "cave list 16"
        prefix : Option<String>,
    },
    ///List available images on dockerhub
    Available {
        ///Optionnal Expression to match, ex : "cave list 16"
        prefix : Option<String>,
    },
    ///Configurate cave
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Activate auto update for stable/testing versions
    EnableAutoUpdate,
    /// Deactivate auto update for stable/testing versions (default)
    DisableAutoUpdate,
    // TODO : uncomment to have registry option
    //  
    // ///Define a personnal registry
    // SetRegistry {
    //     ///Repository 
    //     repo : String,
    //     ///Username
    //     user : String,
    //     ///Personal Access Token (PAT)
    //     token : String
    // },
    // ///Erase the personal registry
    // EraseRegistry,
    ///Enable version usage tracking (default)
    EnableUsageTracking,
    ///Disable version usage tracking
    DisableUsageTracking
}