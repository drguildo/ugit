pub mod base;
pub mod data;
pub mod diff;

pub const UGIT_DIR: &str = ".ugit";

#[derive(Debug)]
pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub message: String,
}

type Tree = Vec<(String, std::ffi::OsString)>;
