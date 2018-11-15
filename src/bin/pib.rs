extern crate pib;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate log;
extern crate env_logger;

use pib::options::{SetupOptions,BuildOptions};
use pib::types::Error;
use structopt::StructOpt;
use log::LevelFilter;


#[derive(Debug,Clone,StructOpt)]
#[structopt(about = "PoA deployment configuration for Parity")]
pub struct Opt {
    /// Set internal log-level
    #[structopt(short="l",long="log",name="level",default_value="info")]
    log_level: LevelFilter,
    /// Command to be performed
    #[structopt(subcommand)]
    cmd: Cmd,
}


#[derive(Debug,Clone,StructOpt)]
enum Cmd {
    /// Initalize a new project
    #[structopt(name = "new")]
    New {
        #[structopt(flatten)]
        setup_options: SetupOptions,
    },
    /// Import entities from an external config file
    #[structopt(name = "import")]
    Import {
        #[structopt(name = "file-path")]
        file_path: String,
    },
    /// Build current project
    #[structopt(name = "build")]
    Build {
        #[structopt(flatten)]
        build_options: BuildOptions,
    },
}


fn main() -> Result<(),Error> {
    let opt = Opt::from_args();
    
    env_logger::Builder::from_default_env()
        .filter_module("pib",opt.log_level)
        .init();
    debug!("executing {:?}",opt.cmd);
    match opt.cmd {
        Cmd::New { setup_options } => {
            pib::setup(setup_options)?;
        },
        Cmd::Import { file_path } => {
            pib::import(file_path)?;
        },
        Cmd::Build { build_options } => {
            pib::build(build_options)?;
        },
    }
    Ok(())
}

