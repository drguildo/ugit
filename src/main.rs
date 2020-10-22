use std::{
    env, fs,
    io::{self, Write},
    process,
};

use clap::{App, Arg, SubCommand};

mod ugit;

fn main() {
    const ABOUT_INIT: &str = "Create a new ugit repository";
    const ABOUT_HASH_OBJECT: &str = "Add the specified file to the object store and print its OID";
    const ABOUT_CAT_FILE: &str = "Print the contents of the file with the specified OID";
    const ABOUT_WRITE_TREE: &str = "Write the current directory to the object store";
    const ABOUT_READ_TREE: &str =
        "Replace the contents of the current directory with the tree with the specified OID";
    const ABOUT_COMMIT: &str = "Commit the current directory";

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
            SubCommand::with_name("commit")
                .arg(
                    Arg::with_name("message")
                        .short("m")
                        .long("message")
                        .takes_value(true)
                        .required(true),
                )
                .about(ABOUT_COMMIT),
        )
        .subcommand(
            SubCommand::with_name("log").arg(Arg::with_name("commit_oid").takes_value(true)),
        )
        .subcommand(
            SubCommand::with_name("checkout").arg(Arg::with_name("commit_oid").takes_value(true)),
        )
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        ugit::data::init();
        process::exit(0);
    }

    // All of the subsequent subcommands need to be run within an existing ugit repository, so exit
    // with an error if the current working directory isn't one.
    exit_if_not_repository();

    if let Some(matches) = matches.subcommand_matches("hash-object") {
        let filename = matches.value_of("filename").unwrap();
        let contents = fs::read(filename).expect("Failed to read file contents");
        let object_hash = ugit::data::hash_object(&contents, "blob");
        io::stdout()
            .write_all(object_hash.as_bytes())
            .expect("Failed to output object ID");
        io::stdout().write(b"\n").expect("Failed to output newline");
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("cat-file") {
        let oid = matches.value_of("oid").unwrap();
        let contents = ugit::data::get_object(oid, None);
        io::stdout()
            .write_all(&contents)
            .expect("Failed to output file data");
        process::exit(0);
    }

    if let Some(_matches) = matches.subcommand_matches("write-tree") {
        let cwd = env::current_dir().expect("Failed to get current working directory");
        ugit::base::write_tree(cwd.as_path());
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("read-tree") {
        let tree_oid = matches.value_of("tree_oid").unwrap();
        ugit::base::read_tree(tree_oid);
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("commit") {
        let message = matches.value_of("message").unwrap();
        ugit::base::commit(message);
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("log") {
        if let Some(commit_oid) = matches.value_of("commit_oid") {
            log(commit_oid);
        } else {
            let head_oid = ugit::data::get_ref("HEAD");
            if head_oid.is_none() {
                eprintln!("No commit OID specified and no HEAD found");
                process::exit(1);
            }
            log(head_oid.unwrap().as_str());
        };
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("checkout") {
        let tree_oid = matches.value_of("commit_oid").unwrap();
        ugit::base::checkout(tree_oid);
        process::exit(0);
    }
}

/// Beginning at the commit with the specified OID, print the commit message and repeatedly do the
/// same for the parent commit, if it exists.
fn log(oid: &str) {
    let mut oid_opt = Some(oid.to_string());
    while oid_opt.is_some() {
        let oid = oid_opt.unwrap();
        let commit = ugit::base::get_commit(&oid);
        println!("commit {}", oid);
        println!("{}\n", commit.message);
        oid_opt = commit.parent;
    }
}

fn exit_if_not_repository() {
    let cwd = env::current_dir().expect("Failed to get current directory");
    if !ugit::base::is_ugit_repository(&cwd) {
        eprintln!("Not a ugit repository.");
        process::exit(1);
    }
}
