use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{base, data, DEFAULT_REPO};

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
            &PathBuf::from(DEFAULT_REPO),
            &refname,
            &data::RefValue {
                symbolic: false,
                value,
            },
            true,
        )
    }
}

pub fn push(remote_path: &Path, ref_name: &str) {
    let default_repo = &PathBuf::from(DEFAULT_REPO);

    // Get refs data.
    let remote_refs = get_remote_refs(remote_path, None);
    let local_ref = data::get_ref(default_repo, ref_name, true)
        .value
        .expect("Ref has no OID");

    // Compute which objects the server doesn't have.
    let known_remote_refs: Vec<&str> = remote_refs
        .values()
        .flatten()
        .filter(|oid| data::object_exists(default_repo, oid))
        .map(AsRef::as_ref)
        .collect();
    let remote_objects = base::get_objects_in_commits(remote_path, known_remote_refs);
    let local_objects = base::get_objects_in_commits(default_repo, vec![&local_ref]);
    let objects_to_push = local_objects.difference(&remote_objects);

    // Push all objects.
    for oid in objects_to_push {
        data::push_object(remote_path, &oid);
    }

    // Update server ref to our value.
    data::update_ref(
        remote_path,
        ref_name,
        &data::RefValue {
            symbolic: false,
            value: Some(local_ref),
        },
        true,
    );
}

fn get_remote_refs(remote_path: &Path, prefix: Option<&str>) -> HashMap<String, Option<String>> {
    let mut result = HashMap::new();
    for (refname, reference) in data::get_refs(remote_path, prefix, true) {
        result.insert(refname, reference.value);
    }
    result
}
