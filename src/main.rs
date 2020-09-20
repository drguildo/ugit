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
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        ugit::data::init();
        std::process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("hash-object") {
        let filename = matches.value_of("filename").unwrap();
        let contents = std::fs::read(filename).expect("Failed to read file contents");
        let object_hash: String = ugit::data::hash_object(&contents);
        println!("{}", object_hash);
        std::process::exit(0);
    }
}
