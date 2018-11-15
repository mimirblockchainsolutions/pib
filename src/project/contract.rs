use mimir_crypto::secp256k1::Address;
use mimir_types::Bytes;
use project::Project;
use types::{Tags,Error};
use util;
use ethabi::{Param,ParamType,Constructor,Token};
use serde::de::{self,Deserialize,Deserializer};
use serde::ser::{Serialize,Serializer};
use toml;
use std::process::Command;
use std::str::FromStr;
use std::path::Path;
use std::{fs,fmt};


/// Source of example/debug contract
pub const EXAMPLE_SOURCE: &str = include_str!("../include/EchoContract.sol");

/// Filename of example/debug contract
pub const EXAMPLE_FILENAME: &str = "EchoContract.sol";

/// Config value of example/debug contract
pub const EXAMPLE_CONFIG: &str = r#"
name = "EchoContract"
addr = "0x0000000000000000000000000000000000003c40"
"#;


/// An argument expected by a contract
#[derive(Debug,Clone)]
pub enum ContractArgument {
    /// Dynamic array of all validator addresses
    AuthorityAddrs,
    /// Dynamic array of matching addresses (matched by tag)
    MatchAddrs(String),
    /// Account address by name
    AccountAddr(String),
    /// Arbitrary address
    Address(Address),
    /// Arbitrary file contents
    Include(String),
}


impl Serialize for ContractArgument {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok,S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}


impl<'de> Deserialize<'de> for ContractArgument {

    fn deserialize<D>(deserializer: D) -> Result<Self,D::Error> where D: Deserializer<'de> {
        let target: util::Either<&str,String> = Deserialize::deserialize(deserializer)?;
        let argbuf: &str = target.as_ref();
        argbuf.parse().map_err(de::Error::custom)
    }
}

impl FromStr for ContractArgument {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        match s.trim() {
            "authority-addrs" => Ok(ContractArgument::AuthorityAddrs),
            arg if arg.contains("::") => {
                let mut split = arg.splitn(2,"::");
                match (split.next(),split.next()) {
                    (Some("match-addrs"),Some(tag)) => {
                        Ok(ContractArgument::MatchAddrs(tag.to_owned()))
                    },
                    (Some("account-addr"),Some(name)) => {
                        Ok(ContractArgument::AccountAddr(name.to_owned()))
                    },
                    (Some("address"),Some(address)) => {
                        let parsed = address.parse()?;
                        Ok(ContractArgument::Address(parsed))
                    },
                    (Some("include"),Some(filename)) => {
                        Ok(ContractArgument::Include(filename.into()))
                    },
                    _=> {
                        let message = format!("unknown contract argument `{}`",s.trim());
                        Err(Error::message(message))
                    }
                }
            },
            other => {
                let message = format!("unknown contract argument `{}`",other);
                Err(Error::message(message))
            }
        }
    }
}


impl fmt::Display for ContractArgument {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContractArgument::AuthorityAddrs => f.write_str("authority-addrs"),
            ContractArgument::MatchAddrs(tag) => {
                write!(f,"match-addrs::{}",tag)
            },
            ContractArgument::AccountAddr(name) => {
                write!(f,"account-addr::{}",name)
            },
            ContractArgument::Address(addr) => {
                write!(f,"address::{}",addr.to_string().trim_left_matches("0x"))
            },
            ContractArgument::Include(file) => {
                write!(f,"include::{}",file)
            },
        }
    }
}


impl ContractArgument {

    fn build_with(&self, project: &Project) -> Result<(Param,Token),Error> {
        match self {
            ContractArgument::AuthorityAddrs => {
                let param_type = ParamType::Array(Box::new(ParamType::Address));
                let param = Param { name: "authority_addrs".into(), kind: param_type };
                let authority_addrs: Vec<Token> = project.iter_nodes().filter_map(|node| {
                    if node.is_authority() {
                        Some(Token::Address(node.account_addr().into_inner().into()))
                    } else {
                        None
                    }
                }).collect();
                let token = Token::Array(authority_addrs);
                Ok((param,token))
            },
            ContractArgument::MatchAddrs(tag) => {
                let param_type = ParamType::Array(Box::new(ParamType::Address));
                let param = Param { name: tag.to_owned(), kind: param_type }; 
                let matches: Vec<Token> = project.iter_accounts().filter_map(|account| {
                    if account.tags().contains(tag) {
                        Some(Token::Address(account.address().into_inner().into()))
                    } else {
                        None
                    }
                }).collect();
                let token = Token::Array(matches);
                Ok((param,token))
            },
            ContractArgument::AccountAddr(name) => {
                let param_type = ParamType::Address;
                let param = Param { name: name.to_owned(), kind: param_type };
                if let Some(account) = project.iter_accounts().find(|a| a.name() == name) {
                    let token = Token::Address(account.address().into_inner().into());
                    Ok((param,token))
                } else {
                    let message = format!("unable to locate include address of `{}` (not found)",name);
                    Err(Error::message(message))
                }
            },
            ContractArgument::Address(address) => {
                let param_type = ParamType::Address;
                let param = Param { name: "address".into(), kind: param_type };
                let token = Token::Address(address.into_inner().into());
                Ok((param,token))
            },
            ContractArgument::Include(filename) => {
                if let Some(buffer) = project.get_template(&filename) {
                    let param_type = ParamType::String;
                    let param = Param { name: "include".into(), kind: param_type };
                    let token = Token::String(buffer.to_owned());
                Ok((param,token))
                } else {
                    let message = format!("unable to include `{}` (not found)",filename);
                    Err(Error::message(message))
                }
            }
        }
    }
}


/// encode contract arguments
fn encode_arguments(project: &Project, args: &[ContractArgument], code: Bytes) -> Result<Bytes,Error> {
    if !args.is_empty() {
        let (mut params, mut tokens) = (Vec::new(),Vec::new());
        for argument in args.iter() {
            let (param,token) = argument.build_with(project)?;
            params.push(param);
            tokens.push(token);
        }
        let constructor = Constructor { inputs: params };
        let encoded = constructor.encode_input(code.into_inner(),&tokens)?;
        Ok(Bytes::from(encoded))
    } else {
        Ok(code)
    }
}


/// Fully specified contract
#[derive(Debug,Clone)]
pub struct Contract {
    pub name: String,
    pub addr: Address,
    pub code: Bytes,
    pub args: Vec<ContractArgument>,
    pub role: Option<ContractRole>,
    pub tags: Tags,
}


impl Contract {

    /// Seed constructor with project-specific arguments if required.
    ///
    pub fn seed_args(&self, project: &Project) -> Result<Option<Bytes>,Error> {
        if !self.args.is_empty() {
            let encoded = encode_arguments(project,&self.args,self.code.clone())?;
            Ok(Some(encoded))
        } else {
            Ok(None)
        }
    }
}


#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractConfig {
    /// Name of contract
    pub name: String,
    
    /// Address of contract
    pub addr: Address,
    
    /// Constructor code (loaded from `contracts/{name}` if unspecified)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub code: Option<Bytes>,
    
    /// Arguments to be seeded (if any)
    #[serde(default)]
    pub args: Vec<ContractArgument>,
    
    /// System role (if any)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub role: Option<ContractRole>,

    /// Arbitrary tags
    #[serde(default)]
    pub tags: Tags,
}


impl ContractConfig {

    pub fn new(name: String, addr: Address) -> Self {
        let (code,args,role,tags) = Default::default();
        Self { name, addr, code, args, role, tags }
    }

    pub fn load_contract(&self, contract_dir: impl AsRef<Path>, no_solc: bool) -> Result<Contract,Error> {
        util::check_name(&self.name)?;
        let name = self.name.to_owned();
        let code: Bytes = match self.code.as_ref() {
            Some(code) => code.to_owned(),
            None => {
                let mut path = contract_dir.as_ref().join(&name);
                if !no_solc {
                    path.set_extension("sol");
                    if path.is_file() {
                        debug!("compiling contract {:?}",path);
                        let exit_status = {
                            let file = path.file_name().expect("name must exist if file exists");
                            let dir = path.parent().unwrap_or(".".as_ref());
                            Command::new("solc")
                                .arg("-o").arg(".")
                                .arg("--abi").arg("--bin")
                                .arg("--overwrite")
                                .arg(&file)
                                .current_dir(dir)
                                .status()?
                        };
                        if exit_status.success() {
                            path.set_extension("bin");
                            fs::read_to_string(&path)?.parse()?
                        } else {
                            let msg = format!("compilation failed for `{}`",path.to_string_lossy());
                            return Err(Error::message(msg));
                        }
                    } else {
                        let msg = format!("unable to locate `{}.sol`",name);
                        return Err(Error::message(msg));
                    }
                } else {
                    path.set_extension("bin");
                    if path.is_file() {
                        debug!("loading existing binary {:?}",path);
                        fs::read_to_string(&path)?.parse()?
                    } else {
                        let msg = format!("unable to locate `{}.bin`",name);
                        return Err(Error::message(msg));
                    }
                }
            }
        };
        let (addr,role,args,tags) = (self.addr,self.role,self.args.clone(),self.tags.clone());
        Ok(Contract { name, addr, code, args, role, tags })
    }
}


impl FromStr for ContractConfig {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let parsed = toml::from_str(s)?;
        Ok(parsed)
    }
}


/// Contract role w/ special meaning to parity
#[derive(Debug,Copy,Clone,Serialize,Deserialize)]
#[serde(untagged)]
pub enum ContractRole {
    Validator(ValidatorContract),
    System(SystemContract)
}


/// Role of validator contract (reporting vs non-reporting)
#[derive(Debug,Copy,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidatorContract {
    /// Simple non-reporting validator set
    ValidatorSetSimple,
    /// Validator set with reporting
    ValidatorSetReporting
}

impl ValidatorContract {

    /// Get the name of the contract role as expected by parity.
    pub fn role_name(&self) -> &'static str {
        match self {
            ValidatorContract::ValidatorSetSimple => "safeContract",
            ValidatorContract::ValidatorSetReporting => "contract",
        }
    }
}

/// Role of system contract (e.g. transaction permissioning).
#[derive(Debug,Copy,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemContract {
    /// Equivalent to `transactionPermissionContract`
    TransactionPermission,
    /// Equivalent to `nodePermissionContract`
    NodePermission,
}


impl SystemContract {

    /// Get the name of the contract role as expected by parity.
    pub fn role_name(&self) -> &'static str {
        match self {
            SystemContract::TransactionPermission => "transactionPermissionContract",
            SystemContract::NodePermission => "nodePermissionContract",
        }
    }
}




