/// Build `peers.txt` file for authorities
use types::{EnodeAddr,Error};
use std::str::FromStr;
use std::fmt;


use config::ConfigModule;
use project::BuildContext;

/// Implementation target for the `ConfigModule` trait.
pub struct Module;


impl ConfigModule for Module {

    fn build(&self, ctx: &mut BuildContext) -> Result<(),Error> {
        let mut peers: Peers = ctx.project_files.templates().get(FILE_NAME)
            .unwrap_or(TEMPLATE).parse()?;
        peers.extend(ctx.project.iter_nodes().map(|n| n.enode_addr()));
        let peer_buff = peers.to_string();
        for node in ctx.project.iter_nodes() {
            ctx.build_files.node(node.name()).config().insert(FILE_NAME,peer_buff.clone());
        }
        Ok(())
    }
}


const FILE_NAME: &'static str = "peers.txt";

const TEMPLATE: &'static str = "";


#[derive(Default,Debug,Clone)]
pub struct Peers {
    inner: Vec<EnodeAddr>
}


impl Peers {

    pub fn add_peer(&mut self, addr: EnodeAddr) {
        self.inner.push(addr);
    }
}


impl Extend<EnodeAddr> for Peers {

    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=EnodeAddr> {
        self.inner.extend(iter)
    }
}


impl FromStr for Peers {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let mut inner = Vec::new();
        for substring in s.split_whitespace() {
            let addr = substring.parse()?;
            inner.push(addr);
        }
        Ok(Self { inner })
    }
}


impl fmt::Display for Peers {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for addr in self.inner.iter() {
            writeln!(f,"{}",addr)?;
        }
        Ok(())
    }
}

