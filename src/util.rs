//! Miscellaneous helpers.
use types::Error;
use ignore::WalkBuilder;
use std::path::{Path,PathBuf};
use std::{io,fs};
use rand::{self,Rng};
use hex;


/// Generate a random alphaneumeric password
pub(crate) fn rand_pass() -> String {
    let pass: String = rand::thread_rng().gen_iter::<char>()
        .filter(char::is_ascii_alphanumeric)
        .take(32)
        .collect();
    pass
}


pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

const IGNORE_FILE: &str = ".pibignore";


pub type FileData = String;

pub type FileName = PathBuf;


pub fn load_dir(path: impl AsRef<Path>) -> impl Iterator<Item=Result<(FileName,FileData),Error>> {
    DirLoader::new(path).load()
}


/// Helper type for recursively loading files.
///
/// ## Example
///
/// ```
/// extern crate pib;
/// 
/// use std::collections::HashMap;
/// use pib::util::DirLoader;
///
/// # fn test() {
/// let files: HashMap<_,_> = DirLoader::new("path/to/files")
///     .git_ignore(true).load()
///     .collect::<Result<_,_>>()
///     .unwrap();
/// 
/// for (name,data) in files.iter() {
///     println!("{:?}: {:?}",name,data);
/// }
/// # }
///
/// # fn main() { }
/// ```
///
#[derive(Debug,Clone)]
pub struct DirLoader<T> {
    target_dir:T,
    strip_prefix: bool,
    git_ignore: bool,
}


impl<T> DirLoader<T> {

    pub fn new(path: T) -> Self {
        Self {
            target_dir: path,
            strip_prefix: true,
            git_ignore: false,
        }
    }

    pub fn strip_prefix(mut self, enabled: bool) -> Self { self.strip_prefix = enabled; self }

    pub fn git_ignore(mut self, enabled: bool) -> Self { self.git_ignore = enabled; self }

    pub fn load(self) -> impl Iterator<Item=Result<(FileName,FileData),Error>> where T: AsRef<Path> {
        WalkBuilder::new(self.target_dir.as_ref()).git_global(self.git_ignore)
            .git_ignore(self.git_ignore).git_exclude(self.git_ignore)
            .add_custom_ignore_filename(IGNORE_FILE).build()
            .filter_map(|rslt| rslt.ok().filter(|entry| entry.path().is_file()))
            .map(move |entry| {
                let filedata = fs::read_to_string(entry.path())?;
                let filename = if self.strip_prefix {
                    entry.path().strip_prefix(self.target_dir.as_ref())?
                } else {
                    entry.path()
                };
                Ok((filename.to_owned(),filedata))
            })
    }
}


#[derive(Debug,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Either<A,B> {
    A(A),
    B(B)
}


impl<A,B,T> AsRef<T> for Either<A,B> where A: AsRef<T>, B: AsRef<T>, T: ?Sized {

    fn as_ref(&self) -> &T {
        match self {
            Either::A(a) => a.as_ref(),
            Either::B(b) => b.as_ref()
        }
    }
}

pub fn save(filepath: impl AsRef<Path>, data: impl AsRef<[u8]>) -> io::Result<()> {
    let path: &Path = filepath.as_ref();
    if let Some(parent) = path.parent() {
        let parent_str = parent.to_str().expect("always valid UTF-8");
        if parent_str.len() > 0 {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path,data.as_ref())
}


pub fn try_save(filepath: impl AsRef<Path>, data: impl AsRef<[u8]>, force: bool) -> Result<(),Error> {
    let path: &Path = filepath.as_ref();
    if force || !path.exists() {
        save(path,data)?;
        Ok(())
    } else {
        let msg = format!("refusing to overwrite `{}` (use --force if this is intentional)",path.to_string_lossy());
        Err(Error::message(msg))
    }
}


pub fn hex_string(bytes: &[u8]) -> String {
    let mut buff = vec![0u8;bytes.len() * 2];
    let _ = hex::as_str(bytes,&mut buff);
    String::from_utf8(buff).expect("always valid UTF-8")
}

pub fn check_name(name: &str) -> Result<(),Error> {
    if name.len() == 0 {
        Err(Error::message("names/paths cannot be empty"))
    } else if name.contains(char::is_whitespace) {
        Err(Error::message(format!("contains whitespace: `{}`",name)))
    } else {
        Ok(())
    }
}


/// Serialize/Deserialize a type using its `Display` and `FromStr` implementations respectively.
pub mod serde_str {
    use serde::de::{self,Deserializer};
    use serde::ser::Serializer;
    use std::fmt::Display;
    use std::str::FromStr;

    /// Serialize any type which implemented `Display`
    ///
    pub fn serialize<T,S>(item: &T, serializer: S) -> Result<S::Ok,S::Error> where S: Serializer, T: Display {
        serializer.collect_str(item)
    }

    /// Deserialize any type which implements `FromStr`
    ///
    pub fn deserialize<'de,T,D>(deserializer: D) -> Result<T,D::Error> where T: FromStr, T::Err: Display, D: Deserializer<'de> {
        let target: Target = de::Deserialize::deserialize(deserializer)?;
        let parsed = target.as_str().parse().map_err(de::Error::custom)?;
        Ok(parsed)
    }


    /// Intermediate deserialization target; prefers `&str` but will successfully
    /// accept `String` (e.g. when deserializing from `serde_json::Value`).
    #[derive(Debug,Serialize,Deserialize)]
    #[serde(untagged)]
    enum Target<'a> {
        Ref(&'a str),
        Own(String),
    }

    impl<'a> Target<'a> {

        fn as_str(&self) -> &str {
            match self {
                Target::Ref(s) => s,
                Target::Own(s) => s,
            }
        }
    }
}
