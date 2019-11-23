use clap::*;

fn main() {
    let matches = App::new("Ana judge program")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("rpc")
                .about("Start a RPC server.")
                .arg(
                    Arg::with_name("threads")
                        .takes_value(true)
                        .value_name("N")
                        .long("threads")
                        .short("N")
                        .help("The max size of the judging thread pool")
                        .env("ANA_THREADS")
                        .default_value("1"),
                )
                .arg(
                    Arg::with_name("address")
                        .takes_value(true)
                        .value_name("ADDRESS")
                        .long("address")
                        .short("l")
                        .help("The listening address")
                        .env("ANA_ADDRESS")
                        .default_value("0.0.0.0"),
                )
                .arg(
                    Arg::with_name("port")
                        .takes_value(true)
                        .value_name("PORT")
                        .long("port")
                        .short("p")
                        .help("The listening port")
                        .env("ANA_PORT")
                        .default_value("8800"),
                ),
        )
        .subcommand(
            SubCommand::with_name("judge")
                .about("Judge a task in a directory.")
                .arg(Arg::with_name("workspace").required(true).takes_value(true)),
        )
        .get_matches();

    unimplemented!()
}
