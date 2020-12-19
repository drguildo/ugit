use std::path::Path;

use super::{data, DEFAULT_REPO};

pub fn fetch(remote_path: &Path) {
    println!("Will fetch the following refs:");

    let mut remote_object_store = remote_path.to_path_buf();
    remote_object_store.push(DEFAULT_REPO);

    for (refname, _) in data::get_refs(&remote_object_store, Some("refs/heads"), true) {
        println!("- {}", refname);
    }
}
