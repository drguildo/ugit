use std::io::Write;

use clap::{App, Arg, SubCommand};

mod ugit;

fn main() {
    let matches = App::new(clap::crate_name!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .subcommand(SubCommand::with_name("init"))
        .subcommand(
            SubCommand::with_name("hash-object").arg(Arg::with_name("filename").required(true)),
        )
        .subcommand(SubCommand::with_name("cat-file").arg(Arg::with_name("oid").required(true)))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        ugit::data::init();
        std::process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("hash-object") {
        let filename = matches.value_of("filename").unwrap();
        let contents = std::fs::read(filename).expect("Failed to read file contents");
        let object_hash: String = ugit::data::hash_object(&contents);
        std::io::stdout()
            .write_all(object_hash.as_bytes())
            .expect("Failed to output object ID");
        std::process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("cat-file") {
        let oid = matches.value_of("oid").unwrap();
        let contents: Vec<u8> = ugit::data::cat_file(oid);
        std::io::stdout()
            .write_all(&contents)
            .expect("Failed to output file data");
        std::process::exit(0);
    }
}
