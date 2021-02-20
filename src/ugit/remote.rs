use std::{collections::HashMap, path::Path};

use super::{base, data};

const REMOTE_REFS_BASE: &str = "refs/heads/";
const LOCAL_REFS_BASE: &str = "refs/remote/";

pub fn fetch(remote_path: &Path) {
    // Get refs from server.
    let refs = get_remote_refs(remote_path, Some(REMOTE_REFS_BASE));
    let commit_oids = refs
        .values()
        .into_iter()
        .filter_map(|v| v.as_deref())
        .collect();

    // Fetch missing objects by iterating and fetching on demand.
    for oid in base::get_objects_in_commits(remote_path, commit_oids) {
        data::fetch_object_if_missing(remote_path, &oid);
    }

    // Update local refs to match server.
    for (remote_name, value) in refs {
        let mut refname = String::from(LOCAL_REFS_BASE);
        refname.push_str(&remote_name.replacen(REMOTE_REFS_BASE, "", 1));
        data::update_ref(
            &refname,
            &data::RefValue {
                symbolic: false,
                value,
            },
            true,
        )
    }
}

fn get_remote_refs(remote_path: &Path, prefix: Option<&str>) -> HashMap<String, Option<String>> {
    let mut result = HashMap::new();
    for (refname, reference) in data::get_refs(remote_path, prefix, true) {
        result.insert(refname, reference.value);
    }
    result
}
