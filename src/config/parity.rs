/// Parse and build parity's main `config.toml` file
use mimir_crypto::secp256k1::{Address,Secret};
use project::node::NodeRole;
use types::Error;
use util;
use std::collections::HashMap;
use std::str::FromStr;
use toml::value::Value;
use toml;


use config::ConfigModule;
use project::{SetupContext,BuildContext};

/// Implementation target for the `ConfigModule` trait.
pub struct Module;


impl ConfigModule for Module {

    fn setup(&self, ctx: &mut SetupContext) -> Result<(),Error> {
        ctx.files.templates_mut().insert(AUTHORITY_FILENAME,AUTHORITY_TEMPLATE);
        ctx.files.templates_mut().insert(INTERFACE_FILENAME,INTERFACE_TEMPLATE);
        Ok(())
    }

    fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
        let authority_template: ParityConfig = ctx.project_files.templates().get(AUTHORITY_FILENAME)
            .unwrap_or(AUTHORITY_TEMPLATE).parse()?;
        let interface_template: ParityConfig = ctx.project_files.templates().get(INTERFACE_FILENAME)
            .unwrap_or(INTERFACE_TEMPLATE).parse()?;
        for node in ctx.project.iter_nodes().filter_map(|n| n.internal()) {
            let mut config = match node.node_role() {
                NodeRole::Authority => authority_template.clone(),
                NodeRole::Interface => interface_template.clone(),
            };
            config.set_network_key(node.network_key());
            config.set_account_addr(node.account_addr());
            ctx.build_files.node(node.name()).config().insert_toml(FILE_NAME,&config)?;
        }
        Ok(())
    }
}



pub const FILE_NAME: &'static str = "config.toml";

pub const AUTHORITY_FILENAME: &'static str = "authority-config.toml";

pub const INTERFACE_FILENAME: &'static str = "interface-config.toml";

pub const AUTHORITY_TEMPLATE: &'static str = include_str!("../include/authority-config.toml");

pub const INTERFACE_TEMPLATE: &'static str = include_str!("../include/interface-config.toml");


/// Parity config file
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ParityConfig {
    /// Network configuration values
    #[serde(default,skip_serializing_if = "HashMap::is_empty")]
    network: HashMap<String,Value>,
    
    /// Mining configuration values
    #[serde(default,skip_serializing_if = "HashMap::is_empty")]
    mining: HashMap<String,Value>,
    
    /// Extra configuration values
    #[serde(flatten)]
    ext: HashMap<String,Value>
}


impl ParityConfig {

    pub fn set_network_key(&mut self, key: Secret) {
        self.network.insert("node_key".into(),util::hex_string(key.as_ref()).into());
    }

    pub fn set_account_addr(&mut self, addr: Address) {
        self.mining.insert("engine_signer".into(),addr.to_string().into());
    }
}


impl FromStr for ParityConfig {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let parity_config = toml::from_str(s)?;
        Ok(parity_config)
    }
}

