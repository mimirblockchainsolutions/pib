use mimir_crypto::secp256k1::{Address,Secret,Signer};
use types::{Tags,Include,EnodeAddr,Error};
use std::net::SocketAddrV4;
use std::path::Path;
use std::fs;
use util;
use rand;
use toml;

#[derive(Debug,Copy,Clone)]
pub enum Node<'a> {
    Internal(&'a InternalNode),
    External(&'a ExternalNode),
}


impl<'a> Node<'a> {

    pub fn name(&self) -> &'a str {
        match self {
            Node::Internal(node) => &node.node_name,
            Node::External(node) => &node.node_name,
        }
    }

    pub fn node_role(&self) -> NodeRole {
        match self {
            Node::Internal(node) => node.node_role(),
            Node::External(node) => node.node_role,
        }
    }

    pub fn account_addr(&self) -> Address {
        match self {
            Node::Internal(node) => node.account_addr(),
            Node::External(node) => node.account_addr,
        }
    }

    pub fn network_addr(&self) -> SocketAddrV4 {
        match self {
            Node::Internal(node) => node.enode_addr().addr,
            Node::External(node) => node.enode_addr.addr,
        }
    }

    pub fn enode_addr(&self) -> EnodeAddr {
        match self {
            Node::Internal(node) => node.enode_addr(),
            Node::External(node) => node.enode_addr,
        }
    }

    pub fn iter_includes(&self) -> impl Iterator<Item=&Include> {
        match self {
            Node::Internal(node) => node.include.iter(),
            Node::External(node) => node.include.iter(),
        }
    }

    pub fn tags(&self) -> &'a Tags {
        match self {
            Node::Internal(node) => &node.tags,
            Node::External(node) => &node.tags,
        }
    }

    pub fn is_authority(&self) -> bool {
        match self.node_role() {
            NodeRole::Authority => true,
            _other => false,
        }
    }

    pub fn is_interface(&self) -> bool {
        match self.node_role() {
            NodeRole::Interface => true,
            _other => false,
        }
    }

    pub fn internal(&self) -> Option<&'a InternalNode> {
        if let Node::Internal(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn external(&self) -> Option<&'a ExternalNode> {
        if let Node::External(node) = self {
            Some(node)
        } else {
            None
        }
    }
}


impl<'a> From<&'a InternalNode> for Node<'a> {

    fn from(node: &'a InternalNode) -> Self {
        Node::Internal(node)
    }
}


impl<'a> From<&'a ExternalNode> for Node<'a> {

    fn from(node: &'a ExternalNode) -> Self {
        Node::External(node)
    }
}


pub type ExternalNode = ExternalNodeConfig;


/// Internally defined node 
#[derive(Debug,Clone)]
pub struct InternalNode {
    node_name: String,
    node_role: NodeRole,
    network_addr: SocketAddrV4, 
    account_pass: String,
    account_signer: Signer,
    network_signer: Signer,
    include: Vec<Include>,
    actors: Vec<String>,
    tags: Tags,
}


impl InternalNode {

    pub fn try_from(config: InternalNodeConfig) -> Result<Self,Error> {
        let account_pass = config.account_pass.unwrap_or_else(util::rand_pass);
        let account_key = config.account_key.unwrap_or_else(rand::random);
        let network_key = config.network_key.unwrap_or_else(rand::random);

        let account_signer = Signer::new(account_key)?;
        let network_signer = Signer::new(network_key)?;

        Ok(Self {
            node_name: config.node_name,
            node_role: config.node_role,
            network_addr: config.network_addr,
            account_pass: account_pass,
            account_signer: account_signer,
            network_signer: network_signer,
            include: config.include.unwrap_or_default(),
            actors: config.actors.unwrap_or_default(),
            tags: config.tags.unwrap_or_default(),
        })
    }

    pub fn name(&self) -> &str {
        &self.node_name
    }

    pub fn iter_actors(&self) -> impl Iterator<Item=&str> {
        self.actors.iter().map(AsRef::as_ref)
    }

    pub fn node_role(&self) -> NodeRole {
        self.node_role
    }

    pub fn account_signer(&self) -> Signer {
        self.account_signer.clone()
    }

    pub fn account_secret(&self) -> Secret {
        self.account_signer.secret()
    }

    pub fn account_addr(&self) -> Address {
        self.account_signer.address()
    }

    pub fn account_pass(&self) -> &str {
        &self.account_pass
    }

    pub fn network_signer(&self) -> Signer {
        self.network_signer.clone()
    }

    pub fn network_key(&self) -> Secret {
        self.network_signer.secret()
    }

    pub fn enode_addr(&self) -> EnodeAddr {
        let address = self.network_addr.into();
        let public = self.network_signer.public();
        EnodeAddr::new(public,address)
    }

    pub fn network_addr(&self) -> SocketAddrV4 {
        self.network_addr
    }
}


pub enum NodeConfigMut<'a> {
    Internal(&'a mut InternalNodeConfig),
    External(&'a mut ExternalNodeConfig),
}


impl<'a> NodeConfigMut<'a> {

    pub fn add_include(&mut self, include: Include) {
        self.includes().push(include);
    }

    pub fn includes(&mut self) -> &mut Vec<Include> {
        match self {
            NodeConfigMut::Internal(config) => config.include.get_or_insert_with(Vec::new),
            NodeConfigMut::External(config) => &mut config.include,
        }
    }
}


impl<'a> From<&'a mut InternalNodeConfig> for NodeConfigMut<'a> {

    fn from(config: &'a mut InternalNodeConfig) -> Self {
        NodeConfigMut::Internal(config)
    }
}


impl<'a> From<&'a mut ExternalNodeConfig> for NodeConfigMut<'a> {

    fn from(config: &'a mut ExternalNodeConfig) -> Self {
        NodeConfigMut::External(config)
    }
}


#[derive(Debug,Clone)]
pub enum NodeConfig {
    Internal(InternalNodeConfig),
    External(ExternalNodeConfig),
}


impl From<InternalNodeConfig> for NodeConfig {

    fn from(config: InternalNodeConfig) -> Self {
        NodeConfig::Internal(config)
    }
}


impl From<ExternalNodeConfig> for NodeConfig {

    fn from(config: ExternalNodeConfig) -> Self {
        NodeConfig::External(config)
    }
}


/// Externally defined node 
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct ExternalNodeConfig {
    node_name: String,
    node_role: NodeRole,
    account_addr: Address,
    enode_addr: EnodeAddr,
    #[serde(default)]
    include: Vec<Include>,
    #[serde(default)]
    tags: Tags,
}


/// Internally defined node 
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct InternalNodeConfig {
    node_name: String,
    #[serde(default)]
    node_role: NodeRole,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    account_pass: Option<String>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    account_key: Option<Secret>,
    network_addr: SocketAddrV4,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    network_key: Option<Secret>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    include: Option<Vec<Include>>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    actors: Option<Vec<String>>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    tags: Option<Tags>,
}


#[derive(Debug,Copy,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeRole {
    /// Actively participates in consensus (mining)
    Authority,
    /// Passively observes consensus (non-mining)
    Interface,
}


impl Default for NodeRole {

    fn default() -> Self { NodeRole::Interface }
}

impl InternalNodeConfig {

    pub fn new(name: String, address: SocketAddrV4, role: NodeRole) -> Self {
        Self {
            node_name: name,
            node_role: role,
            network_addr: address,
            account_pass: None,
            account_key: None,
            network_key: None,
            include: Default::default(),
            actors: Default::default(),
            tags: Default::default(),
        }
    }

    pub fn authority(name: String, address: SocketAddrV4) -> Self {
        Self::new(name,address,NodeRole::Authority)
    }

    pub fn interface(name: String, address: SocketAddrV4) -> Self {
        Self::new(name,address,NodeRole::Interface)
    }
}



#[derive(Debug,Clone)]
pub struct Nodes {
    internal: Vec<InternalNode>,
    external: Vec<ExternalNode>,
}


impl Nodes {

    pub fn try_from(config: NodeConfigs) -> Result<Self,Error> {
        let NodeConfigs { internal, external } = config;
        let internal = internal.into_iter()
            .map(InternalNode::try_from)
            .collect::<Result<_,_>>()?;
        Ok(Self { internal, external })
    }

    pub fn iter(&self) -> impl Iterator<Item=Node> {
        self.internal.iter().map(Node::from).chain(
            self.external.iter().map(Node::from)
        )
    }
}


/// Node configuration
#[derive(Default,Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct NodeConfigs {
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    internal: Vec<InternalNodeConfig>,
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    external: Vec<ExternalNodeConfig>,
}


// TODO: implement duplicate checks for `insert` and `extend` ops.
impl NodeConfigs {

    pub fn insert(&mut self, node: impl Into<NodeConfig>) {
        match node.into() {
            NodeConfig::Internal(config) => self.internal.push(config),
            NodeConfig::External(config) => self.external.push(config),
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=NodeConfigMut> {
        self.internal.iter_mut().map(NodeConfigMut::from).chain(
            self.external.iter_mut().map(NodeConfigMut::from)
        )
    }

    pub fn import(&mut self, other: Self) {
        let Self { internal, external } = other;
        self.internal.extend(internal);
        self.external.extend(external);
    }

    pub fn load_from(filepath: impl AsRef<Path>) -> Result<Self,Error> {
        let raw_file = fs::read_to_string(filepath)?;
        let config = toml::from_str(&raw_file)?;
        Ok(config)
    }
}


