use serde::de::{self,Deserialize,Deserializer};
use serde::ser::{Serialize,Serializer};
use mimir_crypto::secp256k1::Public;
use std::net::SocketAddrV4;
use std::str::FromStr;
use std::{fmt,error};
use hex;


const PREFIX: &'static str = "enode://";

/// ethereum enode address
/// 
/// ```
/// extern crate pib;
/// 
/// use pib::types::EnodeAddr;
///
/// # fn main() {
/// 
///  let raw = "enode://39198951bcf039efa518a0b87e9ea8c9e0eca0e58a8d786c672d83bfb8f7afe0263998ed73411a0004345aaaafab7975c3baea857064c63c004f84ae28001308@172.16.1.30:30303";
///  
///  let enode: EnodeAddr = raw.parse().unwrap();
///  
///  assert_eq!(raw,enode.to_string());
/// # }
/// ```
///
#[derive(Debug,Copy,Clone)]
pub struct EnodeAddr {
    /// public (network) key
    pub public: Public,

    /// external address
    pub addr: SocketAddrV4
}


impl EnodeAddr {

    pub fn new(public: Public, addr: SocketAddrV4) -> Self {
        Self { public, addr }
    }
}



impl fmt::Display for EnodeAddr {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug_assert!(self.public.len() == 64);
        let mut hex_buf = [0u8;128];
        let hex_str = hex::as_str(&self.public,&mut hex_buf);
        f.write_str(PREFIX)?;
        f.write_str(hex_str)?;
        f.write_str("@")?;
        self.addr.fmt(f)
    }
}


impl FromStr for EnodeAddr {

    type Err = ParseEnodeError;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let trimmed = s.trim();
        let (key_str,addr_str) = match trimmed.trim_left_matches(PREFIX) {
            addr if addr.len() + PREFIX.len() == trimmed.len() => {
                let mut split = addr.splitn(2,'@');
                match (split.next(),split.next()) {
                    (Some(key_str),Some(addr_str)) => (key_str,addr_str),
                    _ => return Err(ParseEnodeError),
                }
            },
            _ => return Err(ParseEnodeError),
        };
        let public: Public = key_str.parse()
            .map_err(|_| ParseEnodeError)?;
        let addr: SocketAddrV4 = addr_str.parse()
            .map_err(|_| ParseEnodeError)?;
        Ok(Self { public, addr })
    }
}


/// error during enode parsing
#[derive(Debug,Copy,Clone)]
pub struct ParseEnodeError;


impl ParseEnodeError {

    fn as_str(&self) -> &'static str { "invalid enode address" }
}


impl fmt::Display for ParseEnodeError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str(self.as_str()) }
}

impl error::Error for ParseEnodeError {

    fn description(&self) -> &'static str { self.as_str() }
}


#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(untagged)]
enum Either<A,B> {
    A(A),
    B(B),
}


impl Serialize for EnodeAddr {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let enode_string = self.to_string();
        serializer.serialize_str(&enode_string)
    }
}


impl<'de> Deserialize<'de> for EnodeAddr {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let target: Either<&str,String> = Deserialize::deserialize(deserializer)?;
        let target_str: &str = match target {
            Either::A(ref a) => a,
            Either::B(ref b) => b,
        };
        let enode: EnodeAddr = target_str.parse()
            .map_err(|e| de::Error::custom(e))?;
        Ok(enode)
    }
}

