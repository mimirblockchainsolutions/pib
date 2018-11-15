//! Configuration templating & generation.
//!
pub mod contract;
pub mod compose;
pub mod include;
pub mod scripts;
pub mod parity;
pub mod chain;
pub mod peers; 


pub mod vars {
    use types::Error;
    use mimir_crypto::secp256k1::{Address,Secret};
    use std::collections::HashMap;
    use std::fmt;
    use util;

    use config::ConfigModule;
    use project::BuildContext;

    /// Implementation target for the `ConfigModule` trait.
    pub struct Module;


    impl ConfigModule for Module {

        fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
            let mut shared_vars = Vars::default();
            shared_vars.insert("PROJECT_NAME",ctx.project.project_name());
            for contract in ctx.project.iter_contracts() {
                let key = format!("CONTRACT_{}",contract.name);
                let val = util::hex_string(&contract.addr);
                shared_vars.insert(&key,val);
            }

            let actor_vars = ctx.project.iter_actors().filter_map(|a|a.internal()).map(|a| {
                let vars = account(a.name(),a.address(),a.secret(),a.password());
                (a.name(),vars)
            }).collect::<HashMap<_,_>>();

            // TODO: add additional shared env vars
            for node in ctx.project.iter_nodes().filter_map(|n|n.internal()) {
                let mut node_vars = shared_vars.clone();
                let network_addr = node.network_addr();
                node_vars.insert("NETWORK_HOST",network_addr.ip().to_string());
                node_vars.insert("NETWORK_PORT",network_addr.port().to_string());

                ctx.build_files.node(node.name()).config().insert(NODE_VARS_FILENAME,node_vars.to_string());

                let acct_vars = account(
                    DEFAULT_ACCOUNT_NAME,
                    node.account_addr(),
                    node.account_secret(),
                    node.account_pass(),
                    );
  
                let acct_file = acct_file_name(DEFAULT_ACCOUNT_NAME);
                ctx.build_files.node(node.name()).config().insert(acct_file,acct_vars.to_string());
                
                for name in node.iter_actors() {
                    if let Some(vars) = actor_vars.get(name) {
                        let filename = acct_file_name(name);
                        ctx.build_files.node(node.name()).config().insert(filename,vars.to_string());
                    } else {
                        let msg = format!("No actor named `{}` (expected by node `{}`)",name,node.name());
                        return Err(Error::message(msg));
                    }
                }
            }
            // TODO: Move generation of top-level `.env` file to compose module.  This module
            // should be focused solely on generating node-specific env files.
            let composebuf = format!("COMPOSE_PROJECT_NAME={:?}\n",ctx.project.project_name());
            ctx.build_files.project().insert(".env",composebuf);
            Ok(())
        }
    }

    pub fn acct_file_name(name: &str) -> String { format!("{}/{}.env",ACCOUNT_DIR,name) }

    pub fn account(name: &str, addr: Address, secret: Secret, pass: &str) -> Vars {
        let mut vars = Vars::default();
        vars.insert("ACCOUNT_NAME",name);
        vars.insert("ACCOUNT_SECRET",util::hex_string(&secret));
        vars.insert("ACCOUNT_PASS",pass);
        vars.insert("ACCOUNT_ADDR",util::hex_string(&addr));
        vars
    }


    #[derive(Default,Debug,Clone)]
    pub struct Vars {
        inner: HashMap<String,String>
    }


    impl Vars {

        pub fn insert(&mut self, key: &str, val: impl Into<String>) {
            let name = varname(key);
            self.inner.insert(name,val.into());
        }
    }


    impl fmt::Display for Vars {

        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for (key,val) in self.inner.iter() {
                writeln!(f,"{}='{}'",key,val)?;
            }
            Ok(())
        }
    }


    const ACCOUNT_DIR: &str = "accounts";

    const NODE_VARS_FILENAME: &str = "pib.env";

    const DEFAULT_ACCOUNT_NAME: &str = "account";

    fn varname(suffix: &str) -> String {
        let mut var = format!("{}_{}",util::CRATE_NAME,suffix);
        var.make_ascii_uppercase();
        var
    }
}


pub mod docker {
    use types::Error;
    use config::ConfigModule;
    use project::{SetupContext,BuildContext};

    /// Implementation target for the `ConfigModule` trait.
    pub struct Module;


    impl ConfigModule for Module {

        fn setup(&self, ctx: &mut SetupContext) -> Result<(),Error> {
            ctx.files.templates_mut().insert(FILE_NAME,TEMPLATE);
            Ok(())
        }

        fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
            let dockerfile: &str = ctx.project_files.templates().get(FILE_NAME)
                .unwrap_or(TEMPLATE);
            let mut dockerignore = String::new();
            let ignore_patterns = ctx.project_files.templates().get("dockerignore")
                .unwrap_or_else(|| ctx.project_files.templates().get(".dockerignore").unwrap_or(""))
                .trim().lines().chain(Some(FILE_NAME));
            for pattern in ignore_patterns {
                dockerignore.push_str(pattern.trim());
                dockerignore.push('\n');
            }
            for node in ctx.project.iter_nodes().filter_map(|n| n.internal()) {
                ctx.build_files.node(node.name()).root().insert(FILE_NAME,dockerfile);
                ctx.build_files.node(node.name()).root().insert(".dockerignore",dockerignore.clone());
            }
            Ok(())
        }
    }

    pub const FILE_NAME: &'static str = "Dockerfile";

    pub const TEMPLATE: &'static str = include_str!("../include/Dockerfile");
}


use project::{SetupContext,BuildContext};
use types::Error;


pub const TEMPLATES: &[(&str,&str)] = &[
    (chain::FILE_NAME,chain::TEMPLATE),
    (parity::AUTHORITY_FILENAME,parity::AUTHORITY_TEMPLATE),
    (parity::INTERFACE_FILENAME,parity::INTERFACE_TEMPLATE),
    (docker::FILE_NAME,docker::TEMPLATE),
    (scripts::INIT_FILENAME,scripts::INIT_TEMPLATE)
];


/// Configuration module.
///
pub trait ConfigModule {

    /// Called during initial project setup; 
    #[allow(unused)]
    fn setup(&self, ctx: &mut SetupContext) -> Result<(),Error> {
        Ok(())
    }

    /// 
    #[allow(unused)]
    fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
        Ok(())
    }
}


/// All `ConfigModule` implementers in no particular order
pub const MODULES: &[&dyn ConfigModule] = &[
    &include::Module,
    &chain::Module,
    &contract::Module,
    &compose::Module,
    &parity::Module,
    &peers::Module,
    &scripts::Module,
    &docker::Module,
    &vars::Module,
];


