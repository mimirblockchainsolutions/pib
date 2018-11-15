use serde::de::{Deserializer,Deserialize};
use serde::ser::{Serializer,Serialize};
use types::Error;
use util;
use std::str::FromStr;
use std::fmt;



/// A file-inclusion argument
///
/// ## Example
///
/// ```
/// # extern crate pib;
/// # use pib::types::Include;
/// # fn main() {
/// // Include the file `source/file.txt` at location `dest/file.txt`:
/// let rename: Include = "source/file.txt:dest/file.txt".parse().unwrap();
/// assert_eq!(rename.src(),"source/file.txt");
/// assert_eq!(rename.dst(),"dest/file.txt");
/// assert_eq!(&rename.to_string(),"source/file.txt:dest/file.txt");
///
/// // Include `the contents of `spam/eggs/` at root:
/// assert!("spam/eggs/:/".parse::<Include>().is_ok());
/// 
/// // Include file without changing its path:
/// let simple: Include = "foo/bar.txt".parse().unwrap();
/// assert_eq!(simple.src(),simple.dst());
///
/// # }
/// ```
///
#[derive(Hash,Debug,Clone,PartialEq,Eq)]
pub struct Include {
    src: String,
    dst: Option<String>,
}

impl Include {

    pub fn src(&self) -> &str { &self.src }

    pub fn dst(&self) -> &str { self.dst.as_ref().unwrap_or(&self.src) }
}

impl Serialize for Include {
    
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok,S::Error> where S: Serializer {
        util::serde_str::serialize(self,serializer)
    }
}


impl<'de> Deserialize<'de> for Include {
    
    fn deserialize<D>(deserializer: D) -> Result<Self,D::Error> where D: Deserializer<'de> {
        util::serde_str::deserialize(deserializer)
    }
}

impl fmt::Display for Include {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.dst {
            Some(dst) => write!(f,"{}:{}",self.src,dst),
            None => write!(f,"{}",self.src),
        }
    }
}


impl FromStr for Include {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let mut split = s.split(":").map(|s| s.trim_left_matches("/"));
        match (split.next(),split.next(),split.next()) {
            (Some(src),None,None) => {
                let src = src.into();
                let dst = None;
                Ok(Self { src, dst })
            },
            (Some(src),Some(dst),None) => {
                let src = src.into();
                let dst = Some(dst.into());
                Ok(Self { src, dst })
            },
            _ => {
                let msg = format!("invalid include arg: `{}`",s);
                Err(Error::message(msg))
            }
        }
    }
}

