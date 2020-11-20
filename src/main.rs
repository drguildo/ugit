use std::{
    collections::HashSet,
    env, fs,
    io::{self, Write},
    process,
};

use clap::{App, Arg, SubCommand};

mod ugit;
use ugit::{base, data};

fn main() {
    const ABOUT_INIT: &str = "Create a new ugit repository";
    const ABOUT_HASH_OBJECT: &str = "Add the specified file to the object store and print its OID";
    const ABOUT_CAT_FILE: &str = "Print the contents of the file with the specified OID";
    const ABOUT_WRITE_TREE: &str = "Write the current directory to the object store";
    const ABOUT_READ_TREE: &str =
        "Replace the contents of the current directory with the tree with the specified OID";
    const ABOUT_COMMIT: &str = "Commit the current directory";
    const ABOUT_LOG: &str = "Print the commit history, optionally beginning at the specified OID";
    const ABOUT_CHECKOUT: &str =
        "Restore the working tree to that of the commit with the specified OID";
    const ABOUT_TAG: &str = "Create a reference with the specified name";
    const ABOUT_BRANCH: &str = "List the available branches, or create a new one";

    let matches = App::new(clap::crate_name!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .subcommand(SubCommand::with_name("init").about(ABOUT_INIT))
        .subcommand(
            SubCommand::with_name("hash-object")
                .about(ABOUT_HASH_OBJECT)
                .arg(Arg::with_name("filename").required(true)),
        )
        .subcommand(
            SubCommand::with_name("cat-file")
                .about(ABOUT_CAT_FILE)
                .arg(Arg::with_name("oid").required(true)),
        )
        .subcommand(SubCommand::with_name("write-tree").about(ABOUT_WRITE_TREE))
        .subcommand(
            SubCommand::with_name("read-tree")
                .about(ABOUT_READ_TREE)
                .arg(Arg::with_name("tree_oid").required(true)),
        )
        .subcommand(
            SubCommand::with_name("commit").about(ABOUT_COMMIT).arg(
                Arg::with_name("message")
                    .short("m")
                    .long("message")
                    .takes_value(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("log")
                .about(ABOUT_LOG)
                .arg(Arg::with_name("commit_oid").default_value("@")),
        )
        .subcommand(
            SubCommand::with_name("show").arg(Arg::with_name("commit_oid").default_value("@")),
        )
        .subcommand(
            SubCommand::with_name("checkout")
                .about(ABOUT_CHECKOUT)
                .arg(Arg::with_name("commit").required(true)),
        )
        .subcommand(
            SubCommand::with_name("tag")
                .about(ABOUT_TAG)
                .arg(Arg::with_name("name").required(true))
                .arg(Arg::with_name("oid").default_value("@")),
        )
        .subcommand(SubCommand::with_name("k"))
        .subcommand(
            SubCommand::with_name("branch")
                .about(ABOUT_BRANCH)
                .arg(Arg::with_name("name"))
                .arg(Arg::with_name("start_point").default_value("@")),
        )
        .subcommand(SubCommand::with_name("status"))
        .subcommand(SubCommand::with_name("reset").arg(Arg::with_name("oid").required(true)))
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .get_matches();

    if matches.subcommand_matches("init").is_some() {
        base::init();
        process::exit(0);
    }

    // All of the subsequent subcommands need to be run within an existing ugit repository, so exit
    // with an error if the current working directory isn't one.
    exit_if_not_repository();

    if let Some(matches) = matches.subcommand_matches("hash-object") {
        let filename = matches.value_of("filename").unwrap();
        let contents = fs::read(filename).expect("Failed to read file contents");
        let object_hash = data::hash_object(&contents, "blob");
        io::stdout()
            .write_all(object_hash.as_bytes())
            .expect("Failed to output object ID");
        io::stdout().write(b"\n").expect("Failed to output newline");
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("cat-file") {
        if let Some(oid) = base::get_oid(matches.value_of("oid").unwrap()) {
            let contents = data::get_object(&oid, None);
            io::stdout()
                .write_all(&contents)
                .expect("Failed to output file data");
        }
        process::exit(0);
    }

    if matches.subcommand_matches("write-tree").is_some() {
        let cwd = env::current_dir().expect("Failed to get current working directory");
        base::write_tree(cwd.as_path());
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("read-tree") {
        if let Some(tree_oid) = base::get_oid(matches.value_of("tree_oid").unwrap()) {
            base::read_tree(&tree_oid);
        }
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("commit") {
        let message = matches.value_of("message").unwrap();
        base::commit(message);
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("log") {
        if let Some(commit_oid) = base::get_oid(matches.value_of("commit_oid").unwrap()) {
            log(&commit_oid);
        }
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("show") {
        let oid = base::get_oid(matches.value_of("commit_oid").unwrap());
        show(oid.as_deref());
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("checkout") {
        let commit = matches.value_of("commit").unwrap();
        base::checkout(&commit);
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("tag") {
        let name = matches.value_of("name").unwrap();
        if let Some(oid) = base::get_oid(matches.value_of("oid").unwrap()) {
            base::create_tag(name, &oid);
        }
        process::exit(0);
    }

    if matches.subcommand_matches("k").is_some() {
        k();
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("branch") {
        if let Some(name) = matches.value_of("name") {
            if let Some(start_point) = base::get_oid(matches.value_of("start_point").unwrap()) {
                base::create_branch(name, &start_point);
                println!(
                    "Branch {} created at {}",
                    name,
                    start_point.chars().take(10).collect::<String>()
                );
            }
        } else {
            let current = base::get_branch_name();
            for branch in base::get_branch_names() {
                if let Some(current) = &current {
                    if branch == *current {
                        println!("* {}", branch);
                        continue;
                    }
                }
                println!("{}", branch);
            }
        }
        process::exit(0);
    }

    if matches.subcommand_matches("status").is_some() {
        status();
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("reset") {
        if let Some(oid) = base::get_oid(matches.value_of("oid").unwrap()) {
            base::reset(&oid);
        }
        process::exit(0);
    }
}

fn print_commit(oid: &str, commit: &ugit::Commit, refs: Option<&Vec<String>>) {
    let refs_str = match refs {
        Some(refs) => format!(" ({})", refs.join(", ")),
        None => "".to_owned(),
    };
    println!("commit {}{}", oid, refs_str);
    println!("{}\n", commit.message);
}

/// Beginning at the commit with the specified OID, print the commit message and repeatedly do the
/// same for the parent commit, if it exists.
fn log(oid: &str) {
    let mut oid_to_ref: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (ref_name, ref_value) in data::get_refs(None, true) {
        if let Some(value) = ref_value.value {
            let refs = oid_to_ref.entry(value).or_insert(Vec::new());
            refs.push(ref_name);
        }
    }

    for oid in base::get_commits_and_parents(vec![oid]) {
        let commit = base::get_commit(&oid);
        let refs = oid_to_ref.get(&oid);
        print_commit(&oid, &commit, refs);
    }
}

fn show(oid: Option<&str>) {
    if let Some(oid) = oid {
        let commit = base::get_commit(oid);
        print_commit(oid, &commit, None);
    }
}

fn k() {
    let mut dot = String::new();
    dot.push_str("digraph commits {\n");

    let mut ref_oids: HashSet<String> = HashSet::new();
    for (refname, ref_value) in data::get_refs(None, false) {
        dot.push_str(format!("\"{}\" [shape=note]\n", refname).as_str());
        dot.push_str(
            format!(
                "\"{}\" -> \"{}\"\n",
                refname,
                ref_value.value.clone().unwrap_or("None".to_owned())
            )
            .as_str(),
        );
        if !ref_value.symbolic {
            if let Some(value) = ref_value.value {
                ref_oids.insert(value);
            }
        }
    }

    for oid in base::get_commits_and_parents(ref_oids.iter().map(String::as_str).collect()) {
        let commit = base::get_commit(&oid);
        dot.push_str(
            format!(
                "\"{}\" [shape=box style=filled label=\"{}\"]\n",
                oid,
                oid.chars().take(10).collect::<String>()
            )
            .as_str(),
        );
        if let Some(parent_oid) = commit.parent {
            dot.push_str(format!("\"{}\" -> \"{}\"\n", oid, parent_oid).as_str());
        }
    }

    dot.push_str("}");

    println!("{}", dot);
}

fn status() {
    let head = base::get_oid("@").expect("Failed to get HEAD");
    if let Some(branch_name) = base::get_branch_name() {
        println!("On branch {}", branch_name);
    } else {
        println!(
            "HEAD detached at {}",
            head.chars().take(10).collect::<String>()
        );
    }
}

fn exit_if_not_repository() {
    let cwd = env::current_dir().expect("Failed to get current directory");
    if !base::is_ugit_repository(&cwd) {
        eprintln!("Not a ugit repository.");
        process::exit(1);
    }
}
