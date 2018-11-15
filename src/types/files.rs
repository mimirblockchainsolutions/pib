use types::Error;
use util;
use std::collections::HashMap;
use std::path::{Path,PathBuf};
use serde::{Serialize,Deserialize};
use serde::de::DeserializeOwned;
use serde_json;
use serde_yaml;
use toml;


const TEMPLATE_DIR: &str = "config/templates";

const CONTRACT_DIR: &str = "config/contracts";

const INCLUDE_DIR: &str = "include";


pub type SetupFiles = ProjectFiles;

/// Represents the files of a project directory
///
/// ## Example
///
/// ```
/// extern crate serde_json;
/// extern crate pib;
///
/// use pib::types::ProjectFiles;
///
/// # fn main() {
/// 
/// let mut files = ProjectFiles::default();
///
/// files.templates_mut().insert("hello.txt","Hello there!");
/// 
/// # }
///
/// ```
///
#[derive(Default,Debug,Clone)]
pub struct ProjectFiles {
    templates: Files,
    contracts: Files,
    includes: Files,
}


impl ProjectFiles {

    pub fn templates(&self) -> &Files { &self.templates }

    pub fn contracts(&self) -> &Files { &self.contracts }

    pub fn includes(&self) -> &Files { &self.includes }

    pub fn templates_mut(&mut self) -> &mut Files { &mut self.templates }

    pub fn contracts_mut(&mut self) -> &mut Files { &mut self.contracts }

    pub fn includes_mut(&mut self) -> &mut Files { &mut self.includes }

    pub fn save_to(&self, path: impl AsRef<Path>, force: bool) -> Result<(),Error> {
        let path = path.as_ref();
        let pairings = [
            (TEMPLATE_DIR,&self.templates),
            (CONTRACT_DIR,&self.contracts),
            (INCLUDE_DIR,&self.includes),
        ];
        for (directory,files) in pairings.iter() {
            let fullpath = path.join(directory);
            files.save_to(fullpath,force)?;
        }
        Ok(())
    }

    pub fn load_from(project_dir: impl AsRef<Path>) -> Result<Self,Error> {
        let project_dir = project_dir.as_ref();
        let template_dir = project_dir.join(TEMPLATE_DIR);
        let templates = util::load_dir(template_dir)
            .collect::<Result<_,_>>()?;
        let contract_dir = project_dir.join(CONTRACT_DIR);
        let contracts = util::load_dir(contract_dir)
            .collect::<Result<_,_>>()?;
        let include_dir = project_dir.join(INCLUDE_DIR);
        let includes = util::load_dir(include_dir)
            .collect::<Result<_,_>>()?;
        Ok(Self { templates, contracts, includes })
    }
}


/// Collector representing the output directory of a build process.
///
/// ## Example
///
/// ```
/// extern crate pib;
/// 
/// use pib::types::BuildFiles;
/// use std::time::Duration;
///
/// # fn main() {
/// 
/// // Instantiate an empty set of build files
/// let mut files = BuildFiles::default();
///
/// // Insert a project-level text file named `hello.txt` containing the string `hi there!`.
/// files.project().insert("hello.txt","hi there!");
/// assert_eq!(files.project().get("hello.txt"),Some("hi there!"));
/// 
/// // Insert a toml file specifically for node `alice` named `time.toml` containing a duration.
/// files.node("alice").insert_toml("time.toml",&Duration::new(123,456)).unwrap();
/// assert_eq!(files.node("alice").get("time.toml"),Some("secs = 123\nnanos = 456\n"));
/// # }
///
/// ```
///
#[derive(Default,Debug,Clone)]
pub struct BuildFiles {
    project_files: Files,
    node_files: HashMap<String,NodeFiles>,
}


impl BuildFiles {

    /// Get mutable reference to project-level output files
    pub fn project(&mut self) -> &mut Files { &mut self.project_files }

    /// Get mutable reference to an individual node's output files
    pub fn node(&mut self, name: &str) -> &mut NodeFiles {
        if self.node_files.contains_key(name) {
            self.node_files.get_mut(name).expect("Mapping already contains node")
        } else {
            self.node_files.entry(name.to_string())
                .or_default()
        }
    }

    pub fn iter_project_files(&self) -> impl Iterator<Item=(&Path,&str)> { self.project_files.iter() }

    pub fn iter_nodes(&self) -> impl Iterator<Item=(&str,&NodeFiles)> {
        self.node_files.iter().map(|(name,files)| (name.as_ref(),files))
    }

    pub fn save_to(&self, path: impl AsRef<Path>, force: bool) -> Result<(),Error> {
        let path = path.as_ref();
        self.project_files.save_to(path,force)?;
        for (name,files) in self.iter_nodes() {
            let root_path = path.join(name);
            let config_path = root_path.join(util::CRATE_NAME);
            files.root.save_to(root_path,force)?;
            files.config.save_to(config_path,force)?;
        }
        Ok(())
    }
}


/// Represents the output directory of an individual node
///
#[derive(Default,Debug,Clone)]
pub struct NodeFiles {
    /// Default config directory (most files should go here)
    config: Files,
    /// Root-level configuration (e.g. `Dockerfile`, or user-defined includes)
    root: Files,
}


// TODO: remove these impls once the old build functions are deprecated
impl ::std::ops::Deref for NodeFiles {

    type Target = Files;

    fn deref(&self) -> &Self::Target {
        warn!("Calling `NodeFiles` as `Files` (depreciated)");
        &self.config
    }
}


impl ::std::ops::DerefMut for NodeFiles {

    fn deref_mut(&mut self) -> &mut Self::Target {
        warn!("Calling `NodeFiles` as `Files` (depreciated)");
        &mut self.config
    }
}

impl NodeFiles {

    /// Get mutable reference to config output dir
    pub fn config(&mut self) -> &mut Files { &mut self.config }

    /// Get mutable reference to the root-level output dir
    pub fn root(&mut self) -> &mut Files { &mut self.root }
}


/// Represents a collection of related files
/// 
/// See [BuildFiles](struct.BuildFiles.html) for example usage.
///
#[derive(Default,Debug,Clone)]
pub struct Files {
    inner: HashMap<PathBuf,String>
}


impl Files {


    pub fn iter(&self) -> impl Iterator<Item=(&Path,&str)> {
        self.inner.iter().map(|(path,buff)| (path.as_ref(),buff.as_ref()))
    }

    pub fn get(&self, path: impl AsRef<Path>) -> Option<&str> {
        self.inner.get(path.as_ref()).map(AsRef::as_ref)
    }

    pub fn get_json<'a, T: Deserialize<'a>>(&'a self, path: impl AsRef<Path>) -> Result<Option<T>,Error> {
        if let Some(data) = self.get(path) {
            let deserialized = serde_json::from_str(data)?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    pub fn get_yaml<T: DeserializeOwned>(&self, path: impl AsRef<Path>) -> Result<Option<T>,Error> {
        if let Some(data) = self.get(path) {
            let deserialized = serde_yaml::from_str(data)?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    pub fn get_toml<'a, T: Deserialize<'a>>(&'a self, path: impl AsRef<Path>) -> Result<Option<T>,Error> {
        if let Some(data) = self.get(path) {
            let deserialized = toml::from_str(data)?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&mut self, path: impl Into<PathBuf>, buff: impl Into<String>) -> Option<String> {
        self.inner.insert(path.into(),buff.into())
    }

    pub fn insert_json<T: Serialize>(&mut self, path: impl Into<PathBuf>, data: &T) -> Result<(),Error> {
        let buff = serde_json::to_string_pretty(data)?;
        let _ = self.insert(path,buff);
        Ok(())
    }

    pub fn insert_yaml<T: Serialize>(&mut self, path: impl Into<PathBuf>, data: &T) -> Result<(),Error> {
        let buff = serde_yaml::to_string(data)?;
        let _ = self.insert(path,buff);
        Ok(())
    }

    pub fn insert_toml<T: Serialize>(&mut self, path: impl Into<PathBuf>, data: &T) -> Result<(),Error> {
        let buff = toml::to_string(data)?;
        let _ = self.insert(path,buff);
        Ok(())
    }

    pub fn save_to(&self, path: impl AsRef<Path>, force: bool) -> Result<(),Error> {
        let path = path.as_ref();
        if force || !path.exists() {
            for (name,data) in self.inner.iter() {
                let filepath = path.join(name);
                util::try_save(filepath,data.as_bytes(),force)?;
            }
            Ok(())
        } else {
            let msg = format!("Refusing to overwrite `{}` (use --force to override this behavior)",path.display());
            Err(Error::message(msg))
        }
    }
}


use std::iter::FromIterator;

impl<P,D> FromIterator<(P,D)> for Files where P: Into<PathBuf>, D: Into<String> {

    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=(P,D)> {
        let mut files = Self::default();
        for (path,data) in iter.into_iter() {
            files.insert(path,data);
        }
        files
    }
}

