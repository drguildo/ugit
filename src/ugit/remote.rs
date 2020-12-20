use std::{collections::HashMap, path::Path};

use super::{data, DEFAULT_REPO};

pub fn fetch(remote_path: &Path) {
    println!("Will fetch the following refs:");

    for refname in get_remote_refs(remote_path, Some("refs/heads")).keys() {
        println!("- {}", refname);
    }
}

fn get_remote_refs(remote_path: &Path, prefix: Option<&str>) -> HashMap<String, Option<String>> {
    let mut remote_object_store = remote_path.to_path_buf();
    remote_object_store.push(DEFAULT_REPO);

    let mut result = HashMap::new();
    for (refname, reference) in data::get_refs(&remote_object_store, prefix, true) {
        result.insert(refname, reference.value);
    }
    result
}
