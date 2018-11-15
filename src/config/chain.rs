/// Parse & build `chain.json` specification
use mimir_crypto::secp256k1::Address;
use mimir_types::Bytes;
use types::Error;
use project::contract::{ContractRole,ValidatorContract};
use serde_json::{self,Value};
use std::collections::HashMap;
use std::str::FromStr;


use config::ConfigModule;
use project::{SetupContext,BuildContext};


pub const FILE_NAME: &'static str = "chain.json";

pub const TEMPLATE: &'static str = include_str!("../include/chain.json");


/// Implementation target for the `ConfigModule` trait.
pub struct Module;


impl ConfigModule for Module {

    fn setup(&self, ctx: &mut SetupContext) -> Result<(),Error> {
        ctx.files.templates_mut().insert(FILE_NAME,TEMPLATE);
        Ok(())
    }

    fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
        let mut chain: ChainSpec = ctx.project_files.templates().get(FILE_NAME)
            .unwrap_or(TEMPLATE)
            .parse()?;
        chain.name = ctx.project.project_name().to_owned();
        // insert all genesis accounts, keeping track of whether or
        // not a validator contract was deployed.
        let mut validator_contract = false;
        for account in ctx.project.iter_accounts() {
            let (address,balance) = (account.address(),account.balance());
            if let Some(contract) = account.contract() {
                if let Some(seeded_code) = contract.seed_args(&ctx.project)? {
                    chain.insert_contract(address,&seeded_code,balance,contract.role)?;
                } else {
                    chain.insert_contract(address,&contract.code,balance,contract.role)?;
                }
                if let Some(ContractRole::Validator(_)) = contract.role {
                    validator_contract = true;
                }
            } else {
                chain.insert_account(address,balance,None)?;
            }
        }
        // if no validator contract was deployed, insert validator
        // list instead
        if !validator_contract {
            let addrs: Vec<_> = ctx.project.iter_nodes().filter(|n| n.is_authority())
                .map(|n| n.account_addr()).collect();
            chain.set_validator_list(&addrs);
        }
        for node in ctx.project.iter_nodes() {
            ctx.build_files.node(node.name()).config().insert_json(FILE_NAME,&chain)?;
        }
        Ok(())
    }
}


/// Chain specification file
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct ChainSpec {
    /// Name of chain
    pub name: String,

    /// Consensus engine description
    pub engine: HashMap<String,Value>,

    /// Genesis block params
    pub genesis: HashMap<String,Value>,

    /// General chain params
    pub params: HashMap<String,Value>,

    /// Genesis accounts
    pub accounts: HashMap<Address,Value>,
}


impl FromStr for ChainSpec {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let chain_spec = serde_json::from_str(s)?;
        Ok(chain_spec)
    }
}


impl ChainSpec {

    pub fn new() -> Self {
        TEMPLATE.parse().expect("defaults must deserialize")
    }

    /// Insert validator set contract (alternative to validator list)
    pub fn set_validator_contract(&mut self, role: ValidatorContract, addr: Address) {
        let try_insert = |spec: &mut Value| -> Option<()> {
            let _ = spec.get_mut("params")?
                .get_mut("validators")?
                .as_object_mut()?
                .insert(role.role_name().into(),addr.to_string().into());
            Some(())
        };
        for (_,spec) in self.engine.iter_mut() {
            let _ = try_insert(spec);
        }
    }

    /// Insert list of validator addresses (alternative to validator contract)
    pub fn set_validator_list(&mut self, addrs: &[Address]) {
        let try_insert = |spec: &mut Value| -> Option<()> {
            let vlist = spec.get_mut("params")?
                .get_mut("validators")?
                .as_object_mut()?
                .entry("list")
                .or_insert(Value::Array(Vec::new()))
                .as_array_mut()?;
            for address in addrs.iter() {
                vlist.push(address.to_string().into());
            }
            Some(())
        };
        for (_,spec) in self.engine.iter_mut() {
            let _ = try_insert(spec);
        }
    }

    /// Insert a genesis account
    pub fn insert_account(&mut self, addr: Address, balance: u64, code: Option<&Bytes>) -> Result<(),Error> {
        if !self.accounts.contains_key(&addr) {
            let spec = if let Some(constructor) = code {
                json!({
                    "balance": balance.to_string(),
                    "constructor": constructor
                })
            } else {
                json!({"balance": balance.to_string()})
            };
            let _ = self.accounts.insert(addr,spec);
            Ok(())
        } else {
            let message = format!("multiple accounts specified for `{:?}`",addr);
            Err(Error::message(message))
        }
    }


    /// insert a genesis contract
    pub fn insert_contract(&mut self, addr: Address, code: &Bytes, balance: u64, role: Option<ContractRole>) -> Result<(),Error> {
        self.insert_account(addr,balance,Some(code))?;
        match role {
            Some(ContractRole::Validator(role)) => {
                self.set_validator_contract(role,addr);
                Ok(())
            },
            Some(ContractRole::System(role)) => {
                let role_name = role.role_name().into();
                let address = addr.to_string().into();
                self.params.insert(role_name,address);
                Ok(())
            },
            None => Ok(())
        }
    }
}

