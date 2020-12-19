pub mod base;
pub mod data;
pub mod diff;
pub mod remote;

pub const DEFAULT_REPO: &str = ".ugit";

#[derive(Debug)]
pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub message: String,
}

type Tree = Vec<(String, std::ffi::OsString)>;
