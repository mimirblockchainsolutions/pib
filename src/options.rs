//use structopt::StructOpt;
use std::path::PathBuf;


/// Command-line options for setup phase
#[derive(Debug,Clone,StructOpt)]
pub struct SetupOptions {
    #[structopt(name = "project-name")]
    pub project_name: String,
    /// Number of authority nodes to generate
    #[structopt(name = "authority-count", long = "authorities",default_value= "3")]
    pub authorities: u16,
    /// Number of interface nodes to generate
    #[structopt(name = "interface-count", long = "interfaces",default_value= "1")]
    pub interfaces: u16,
    /// Number of actor accounts to generate
    #[structopt(name = "actor-count",long = "actors",default_value="0")]
    pub actors: u16,
    #[structopt(long = "no-examples")]
    /// Do not generate examples
    pub no_examples: bool,
    /// Overwrite existing files
    #[structopt(long = "force")]
    pub force: bool,
}


/// Command-line options for build phase
#[derive(Debug,Clone,StructOpt)]
pub struct BuildOptions {
    /// Output directory
    #[structopt(name = "dir", long = "output", default_value = "output")]
    #[structopt(parse(from_os_str))]
    pub output_dir: PathBuf,
    /// Do not invoke solc
    #[structopt(long = "no-solc")]
    pub no_solc: bool, 
    /// Overwrite existing files
    #[structopt(long = "force")]
    pub force: bool,
}

