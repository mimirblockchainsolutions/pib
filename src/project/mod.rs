//! Top-level project configuration.
//!
pub mod node;
pub mod account;
pub mod actor;
pub mod contract;


pub const CONTRACT_DIR: &'static str = "config/contracts";

pub const TEMPLATE_DIR: &'static str = "include";

pub const PROJECT_FILE: &'static str = concat!(env!("CARGO_PKG_NAME"),".toml");


use std::net::Ipv4Addr;
use options::{SetupOptions,BuildOptions};
use project::contract::{ContractConfig,Contract};
use project::account::Account;
use project::node::{
    Node,
    NodeRole,
    NodeConfigs,
    Nodes,
    InternalNodeConfig
};
use project::actor::{
    Actor,
    ActorConfigs,
    InternalActorConfig,
    Actors,
};
use util;
use types::Error;
use std::collections::HashMap;
use std::net::SocketAddrV4;
use std::path::Path;
use std::fs;
use toml;

use types::{ProjectFiles,BuildFiles,SetupFiles};



#[derive(Debug)]
pub struct ProjectContext {
    project_config: ProjectConfig,
    project_files: ProjectFiles,
    build_files: Option<BuildFiles>,
    project: Option<Project>,
}


impl ProjectContext {

    pub fn new(project_config: ProjectConfig) -> Self {
        let (project_files,build_files,project) = Default::default();
        Self { project_config, project_files, build_files, project }
    }

    pub fn setup_context<'a>(&'a mut self, options: &'a SetupOptions) -> SetupContext<'a> {
        SetupContext {
            project: &mut self.project_config,
            options: options,
            files: &mut self.project_files,
        }
    }

    pub fn build_context<'a>(&'a mut self, options: &'a BuildOptions) -> Result<BuildContext<'a>,Error> {
        let project = if self.project.is_some() {
            self.project.as_mut().expect("Project must exist")
        } else {
            let project = Project::try_from(self.project_config.clone(),options.no_solc)?;
            self.project.get_or_insert(project)
        };
        let project_files = &self.project_files;
        let build_files = self.build_files.get_or_insert_with(Default::default);
        Ok(BuildContext { project, options, project_files, build_files })
    }

    pub fn load_from(project_dir: impl AsRef<Path>) -> Result<Self,Error> {
        let project_dir = project_dir.as_ref();
        let config_path = project_dir.join(PROJECT_FILE);
        let project_config = ProjectConfig::load_from(config_path)?;
        let project_files = ProjectFiles::load_from(project_dir)?;
        let (build_files,project) = Default::default();
        Ok(Self { project_config, project_files, build_files, project })
    }
}


#[derive(Debug)]
pub struct SetupContext<'a> {
    pub project: &'a mut ProjectConfig,
    pub options: &'a SetupOptions,
    pub files: &'a mut SetupFiles,
}


impl<'a> SetupContext<'a> {

    pub fn save_to(&self, output_dir: impl AsRef<Path>) -> Result<(),Error> {
        let output_dir = output_dir.as_ref();
        let project_dir = output_dir.join(&self.project.project_name());
        self.files.save_to(&project_dir,self.options.force)?;
        let project_file = project_dir.join(PROJECT_FILE);
        self.project.save_to(project_file)?; 
        Ok(())
    }
}


#[derive(Debug)]
pub struct BuildContext<'a> {
    pub project: &'a Project,
    pub options: &'a BuildOptions,
    pub project_files: &'a ProjectFiles,
    pub build_files: &'a mut BuildFiles,
}


impl<'a> BuildContext<'a> {

    pub fn save_to(&self, output_dir: impl AsRef<Path>) -> Result<(),Error> {
        self.build_files.save_to(output_dir,self.options.force)?;
        Ok(())
    }
}


/// Basic project-level information
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct ProjectInfo {
    project_name: String,
}


impl ProjectInfo {

    pub fn new(name: String) -> Self {
        Self { project_name: name }
    }
}


#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct DockerComposeConfig {
    pub gateway_addr: Ipv4Addr,
    pub expose_iface: bool,
}


impl Default for DockerComposeConfig {

    fn default() -> Self {
        Self { gateway_addr: Ipv4Addr::new(10,0,0,1), expose_iface: true }
    }
}


#[derive(Default,Debug,Clone)]
pub struct Contracts(Vec<Contract>);


impl Contracts { 

    pub fn iter(&self) -> impl Iterator<Item=&Contract> {
        self.0.iter()
    }
}


#[derive(Default,Debug,Clone,Serialize,Deserialize)]
pub struct ContractConfigs(Vec<ContractConfig>);


impl ContractConfigs {

    pub fn import(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    pub fn try_load(&self,contract_dir: impl AsRef<Path>, no_solc: bool) -> Result<Contracts,Error> {
        let contracts = self.0.iter()
            .map(|config| config.load_contract(contract_dir.as_ref(),no_solc))
            .collect::<Result<_,_>>()?;
        Ok(Contracts(contracts))
    }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn insert(&mut self, config: ContractConfig) {
        self.0.push(config);
    }
}


#[derive(Debug,Clone)]
pub struct Project {
    project_info: ProjectInfo,
    docker_compose: Option<DockerComposeConfig>,
    nodes: Nodes,
    actors: Actors,
    contracts: Contracts,
    templates: HashMap<String,String>,
}


impl Project {

    pub fn try_from(config: ProjectConfig, no_solc: bool) -> Result<Self,Error> {
        let nodes = Nodes::try_from(config.nodes)?;
        let actors = Actors::try_from(config.actors)?;
        let contracts = config.contracts.try_load(CONTRACT_DIR,no_solc)?;
        let templates = load_templates(TEMPLATE_DIR)?;
        Ok(Self {
            project_info: config.project_info,
            docker_compose: config.docker_compose,
            nodes: nodes,
            actors: actors,
            contracts: contracts,
            templates: templates
        })
    }

    pub fn project_name(&self) -> &str {
        &self.project_info.project_name
    }

    pub fn compose_config(&self) -> Option<&DockerComposeConfig> {
        self.docker_compose.as_ref()
    }

    pub fn get_template(&self, name: &str) -> Option<&str> {
        self.templates.get(name).map(AsRef::as_ref)
    }

    // TODO: remove in favor of using `iter_nodes` directly
    pub fn iter_authorities(&self) -> impl Iterator<Item=Node> {
        self.iter_nodes().filter(|n| n.is_authority())
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item=Node> {
        self.nodes.iter()
    }

    pub fn iter_actors(&self) -> impl Iterator<Item=Actor> {
        self.actors.iter()
    }

    pub fn iter_contracts(&self) -> impl Iterator<Item=&Contract> {
        self.contracts.iter()
    }

    pub fn iter_accounts(&self) -> impl Iterator<Item=Account> {
        self.iter_nodes().map(From::from).chain(
            self.iter_actors().map(From::from).chain(
                self.iter_contracts().map(From::from)
            )
        )
    }
}


fn load_templates(template_dir: impl AsRef<Path>) -> Result<HashMap<String,String>,Error> {
    if template_dir.as_ref().is_dir() {
        let mut collector = HashMap::new();
        for entry in fs::read_dir(template_dir)? {
            let path = entry?.path();
            if path.is_file() {
                let name = path.file_name().expect("path is file").to_str()
                    .ok_or_else(||Error::message("template names must be UTF-8"))?
                    .to_owned();
                let buff = fs::read_to_string(&path)?;
                collector.insert(name,buff);
            }
        }
        Ok(collector)
    } else {
        Ok(Default::default())
    }
}


#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct ProjectConfig {
    project_info: ProjectInfo,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    docker_compose: Option<DockerComposeConfig>,
    #[serde(rename = "node",default)]
    nodes: NodeConfigs,
    #[serde(rename = "actor",default,skip_serializing_if = "ActorConfigs::is_empty")]
    actors: ActorConfigs,
    #[serde(rename = "contract",default,skip_serializing_if = "ContractConfigs::is_empty")]
    contracts: ContractConfigs
}


impl ProjectConfig {

    pub fn new(name: String, authority_count: u16, interface_count: u16, actor_count: u16) -> Self {
        let project_info = ProjectInfo::new(name);
        let compose = DockerComposeConfig::default();
        let gateway: u32 = compose.gateway_addr.into();
        let start_addr = gateway + 1;
        let mut nodes = NodeConfigs::default();
        let addrs = (0u32..).into_iter().map(|offset| {
            let ip_addr = Ipv4Addr::from(start_addr + offset);
            SocketAddrV4::new(ip_addr,30303)
        });
        let roles = (0..authority_count).into_iter().map(|_|NodeRole::Authority)
            .chain((0..interface_count).into_iter().map(|_|NodeRole::Interface));
        for (index,(addr,role)) in addrs.zip(roles).enumerate() {
            let name = format!("node-{}",index);
            let node = InternalNodeConfig::new(name,addr,role);
            nodes.insert(node);
        }
        let mut actors = ActorConfigs::default();
        for index in 0..actor_count {
            let name = format!("actor-{}",index);
            let actor = InternalActorConfig::new(name);
            actors.insert(actor);
        }
        let docker_compose = Some(compose);
        let contracts = Default::default();
        Self { project_info, docker_compose, nodes, actors, contracts }
    }

    pub fn project_name(&self) -> &str { &self.project_info.project_name }

    pub fn nodes_mut(&mut self) -> &mut NodeConfigs { &mut self.nodes }

    pub fn import(&mut self, other: Self) {
        let Self { nodes, actors, contracts, docker_compose, .. } = other;
        if docker_compose.is_some() { self.docker_compose = docker_compose; }
        self.nodes.import(nodes);
        self.actors.import(actors);
        self.contracts.import(contracts);
    }

    pub fn insert_contract(&mut self, contract: ContractConfig) {
        self.contracts.insert(contract)
    }


    pub fn load_from(filepath: impl AsRef<Path>) -> Result<Self,Error> {
        let raw_file = fs::read_to_string(filepath)?;
        let config = toml::from_str(&raw_file)?;
        Ok(config)
    }

    pub fn save_to(&self, filepath: impl AsRef<Path>) -> Result<(),Error> {
        let serialized = toml::to_string(self)?;
        util::save(filepath,&serialized)?;
        Ok(())
    }
}

