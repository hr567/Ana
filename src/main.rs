use std::io;
use std::net::IpAddr;

use clap::*;

fn main() -> io::Result<()> {
    let matches = App::new("Ana judge program")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
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
        )
        .get_matches();

    let threads: usize = matches
        .value_of("threads")
        .expect("Failed to get threads argument")
        .parse()
        .expect("`threads` argument is invalid");
    let address: IpAddr = matches
        .value_of("address")
        .expect("Failed to get address argument")
        .parse()
        .expect("`address` argument is invalid");
    let port: u16 = matches
        .value_of("port")
        .expect("Failed to get port argument")
        .parse()
        .expect("`port` argument is invalid");

    ana::start_rpc_server(address, port, threads);

    Ok(())
}
