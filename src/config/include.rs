use types::Error;
use config::ConfigModule;
use project::{SetupContext,BuildContext};
use std::path::Path;


const EXAMPLE_FILENAME: &str = "example/include.md";

const EXAMPLE_INCLUDE: &str = "example/:include-example/";

const EXAMPLE_FILEDATA: &str = include_str!("../include/example-include.md");

/// Implementation target for the `ConfigModule` trait.
pub struct Module;

impl ConfigModule for Module {

    fn setup(&self, ctx: &mut SetupContext) -> Result<(),Error> {
        if !ctx.options.no_examples {
            ctx.files.includes_mut().insert(EXAMPLE_FILENAME,EXAMPLE_FILEDATA);
            for mut node in ctx.project.nodes_mut().iter_mut().take(1) {
                let include = EXAMPLE_INCLUDE.parse().expect("Example include must parse");
                node.add_include(include);
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
        for node in ctx.project.iter_nodes() {
            for include in node.iter_includes() {
                if include.src().ends_with("/") {
                    let mut match_count = 0u32;
                    for (name,data) in ctx.project_files.includes().iter() {
                        if name.starts_with(include.src()) {
                            match_count += 1;
                            let suffix = name.strip_prefix(include.src())?;
                            let path = Path::new(include.dst()).join(suffix);
                            ctx.build_files.node(node.name()).root().insert(path,data.to_owned());
                        }
                    }
                    if match_count == 0 {
                        warn!("No matches found for `{}`",include.src());
                    } else {
                        debug!("{} matches found for `{}`",match_count,include.src());
                    }
                } else {
                    if let Some(data) = ctx.project_files.includes().get(include.src()) {
                        ctx.build_files.node(node.name()).root().insert(include.dst(),data.to_owned());
                    } else {
                        let msg = format!("failed to include `{}` for `{}` (file not found)",include.src(),node.name());
                        return Err(Error::message(msg));
                    }
                }
            }
        }
        Ok(())
    }
}

