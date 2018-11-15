//! Build `docker-compose.yml` for project deployments
use project::node::NodeRole;
use types::Error;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::str::FromStr; 
use serde_yaml::Value;
use serde_yaml;

use config::ConfigModule;
use project::BuildContext;
use util;

/// Implementation target for the `ConfigModule` trait.
pub struct Module;


impl ConfigModule for Module {

    fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
        if let Some(config) = ctx.project.compose_config() {
            let mut compose = if let Some(template) = ctx.project_files.templates().get(FILE_NAME) {
                let compose = template.parse()?;
                compose
            } else {
                DockerCompose::default()
            };
            let mut expose_iface = config.expose_iface;
            let network_name = format!("{}-net",ctx.project.project_name());
            for node in ctx.project.iter_nodes() {
                let service_name = node.name().to_string();
                let state_volume = format!("{}-state",service_name);
                let base_config = ServiceConfig::default()
                    .build_ctx(service_name.as_str())
                    //.restart_policy("always")
                    .volume(format!("{}:/{}/state",state_volume,util::CRATE_NAME))
                    .ip_addr(network_name.as_str(),node.network_addr().ip().to_owned());
                compose.add_volume(state_volume);
                let service_config = match node.node_role() {
                    NodeRole::Authority => {
                        // TODO: decide on any authority-specific
                        // configs if any (e.g. tags).
                        base_config
                    },
                    NodeRole::Interface => {
                        if expose_iface {
                            expose_iface = false;
                            info!("exposing local RPC for interface `{}`",node.name());
                            base_config.local_port(8545).local_port(8546)
                        } else {
                            base_config
                        }
                    },
                };
                compose.add_service(service_name,service_config);
            }
            compose.add_network(network_name,config.gateway_addr)?;
            ctx.build_files.project().insert_yaml(FILE_NAME,&compose)
        } else {
            Ok(())
        }
    }
}


const FILE_NAME: &'static str = "docker-compose.yml";


/// Configuration of a service
#[derive(Default,Debug,Clone,Serialize,Deserialize)]
struct ServiceConfig {
    #[serde(default,skip_serializing_if = "HashMap::is_empty")]
    build: HashMap<String,Value>,
    //#[serde(default,skip_serializing_if = "Option::is_none")]
    //restart: Option<String>,
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    volumes: Vec<String>,
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    ports: Vec<String>,
    #[serde(default,skip_serializing_if = "HashMap::is_empty")]
    networks: HashMap<String,HashMap<String,Value>>,
    #[serde(flatten)]
    ext: HashMap<String,Value>,
}


impl ServiceConfig {

    pub fn build_ctx(mut self, ctx: impl Into<String>) -> Self {
        self.build.insert("context".to_owned(),Value::String(ctx.into())); self
    }

    //pub fn restart_policy(mut self, policy: impl Into<String>) -> Self { self.restart = Some(policy.into()); self }

    pub fn volume(mut self, volume: impl Into<String>) -> Self { self.volumes.push(volume.into()); self }

    pub fn local_port(mut self, port: u16) -> Self { self.ports.push(format!("127.0.0.1:{0}:{0}",port)); self }

    pub fn ip_addr(mut self, network: impl Into<String>, gateway: Ipv4Addr) -> Self {
        let key = "ipv4_address".to_string();
        let val = Value::String(gateway.to_string());
        self.networks.entry(network.into())
            .or_default()
            .insert(key,val);
        self
    }
}


/// docker-compose config file
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerCompose {
    #[serde(default = "version_two")]
    version: String,
    #[serde(default)]
    services: HashMap<String,ServiceConfig>,
    #[serde(default)]
    networks: HashMap<String,HashMap<String,HashMap<String,Value>>>,
    #[serde(default)]
    volumes: HashMap<String,HashMap<String,Value>>,
}


fn version_two() -> String { String::from("2.0") }


impl Default for DockerCompose {

    fn default() -> Self {
        let version = version_two();
        let (services,networks,volumes) = Default::default();
        Self { version, services, networks, volumes }
    }
}


impl FromStr for DockerCompose {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let compose = serde_yaml::from_str(s)?;
        Ok(compose)
    }
}

impl DockerCompose {

    fn add_service(&mut self, name: String, config: ServiceConfig) {
        self.services.insert(name,config);
    }

    fn add_network(&mut self, name: String, gateway: Ipv4Addr) -> Result<(),Error> {
        let addr_space = Some(("gateway",gateway.to_string())).into_iter()
            .chain(Some(("subnet",format!("{}/24",gateway))))
            .map(|(k,v)| (Value::from(k),Value::from(v)))
            .collect();
        self.networks.entry(name).or_default()
            .entry("ipam".to_string()).or_default()
            .entry("config".to_string())
            .or_insert_with(|| Value::Sequence(vec![])).as_sequence_mut()
            .ok_or_else(|| Error::message("field `config` must be sequence"))?
            .push(Value::Mapping(addr_space));
        Ok(())
    }

    fn add_volume(&mut self, name: String) {
        self.volumes.insert(name,Default::default());
    }
}

