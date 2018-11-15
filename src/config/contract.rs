use project::SetupContext;
use config::ConfigModule;
use types::Error;

/// Source of example/debug contract
pub const EXAMPLE_SOURCE: &str = include_str!("../include/EchoContract.sol");

/// Filename of example/debug contract
pub const EXAMPLE_FILENAME: &str = "EchoContract.sol";

/// Config value of example/debug contract
pub const EXAMPLE_CONFIG: &str = r#"
name = "EchoContract"
addr = "0x0000000000000000000000000000000000003c40"
"#;


/// Implementation target for the `ConfigModule` trait.
pub struct Module;

impl ConfigModule for Module {

    fn setup(&self, ctx: &mut SetupContext) -> Result<(),Error> {
        if !ctx.options.no_examples {
            ctx.files.contracts_mut().insert(EXAMPLE_FILENAME,EXAMPLE_SOURCE);
            let config = EXAMPLE_CONFIG.parse().expect("example config must parse");
            ctx.project.insert_contract(config);
            Ok(())
        } else {
            Ok(())
        }
    }
}

