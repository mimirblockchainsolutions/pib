/// Misc helper scripts
use types::Error;
use std::fmt::Write;


use config::ConfigModule;
use project::{SetupContext,BuildContext};

/// Implementation target for the `ConfigModule` trait.
pub struct Module;


impl ConfigModule for Module {

    fn setup(&self, ctx: &mut SetupContext) -> Result<(),Error> {
        ctx.files.templates_mut().insert(INIT_FILENAME,INIT_TEMPLATE);
        Ok(())
    }

    fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
        let init_script: &str = ctx.project_files.templates().get(INIT_FILENAME)
            .unwrap_or(INIT_TEMPLATE);
        for node in ctx.project.iter_nodes().filter_map(|n| n.internal()) {
            ctx.build_files.node(node.name()).config().insert(INIT_FILENAME,init_script);
        }
        Ok(())
    }
}


pub const INIT_FILENAME: &'static str = "init.sh";

pub const INIT_TEMPLATE: &'static str  = include_str!("../include/init.sh");


const BUILD_HEADER: &str = 
r##"#!/bin/bash

set -e
"##;



pub fn make_build_script(image_names: impl IntoIterator<Item=impl AsRef<str>>) -> Result<String,Error> {
    let mut buf = String::from(BUILD_HEADER);
    for name in image_names.into_iter() {
        writeln!(buf,"cd {0} && docker build --tag {0} . && cd ..",name.as_ref())?;
    }
    Ok(buf)
}

