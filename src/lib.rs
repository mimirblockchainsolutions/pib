//! Command-line utility for easy Proof of Authority deployments with Parity.
//!
extern crate mimir_crypto;
extern crate mimir_types;
extern crate hex_core as hex;
#[macro_use]
extern crate structopt;
extern crate ignore;
extern crate ethabi;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde_yaml;
extern crate serde;
extern crate toml;
extern crate rand;
#[macro_use]
extern crate log;

pub mod project;
pub mod options;
pub mod config;
pub mod types;
pub mod util;


use std::path::Path;
use types::Error;


use project::{ProjectContext,ProjectConfig};
use options::{SetupOptions,BuildOptions};


pub fn setup(options: SetupOptions) -> Result<(),Error> {

    let project_config = ProjectConfig::new(
        options.project_name.to_string(),
        options.authorities,
        options.interfaces,
        options.actors,
        );

    let mut ctx = ProjectContext::new(project_config);

    let mut setup = ctx.setup_context(&options);

    // Apply all configuration modules to context
    for module in config::MODULES.iter() {
        module.setup(&mut setup)?;
    }

    setup.save_to(".")?;

    Ok(())
}


pub fn build(options: BuildOptions) -> Result<(),Error> {
    let mut ctx = ProjectContext::load_from(".")?;

    let mut build = ctx.build_context(&options)?;

    for module in config::MODULES.iter() {
        module.build(&mut build)?;
    }

    build.save_to(&options.output_dir)?;

    Ok(())
}


pub fn import(config_path: impl AsRef<Path>) -> Result<(),Error> {
    let mut project_config = project::ProjectConfig::load_from(project::PROJECT_FILE)?;

    let import_config = project::ProjectConfig::load_from(config_path)?;

    project_config.import(import_config);

    project_config.save_to(project::PROJECT_FILE)?;

    Ok(())
}

