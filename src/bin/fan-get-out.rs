use clap::{App, Arg};
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

fn main() {
    // Parse command line parameters:
    let matches = App::new("Get current output value of fand")
        .version("0.1")
        .author("")
        .about(
            "Command line client to retrieve current output of the fand control loop. ONLY WORKS
            WITH DEFAULT CONFIG",
        )
        .arg(
            Arg::with_name("SOCKET")
                .help("Path to the socket to connect to.")
                .required(true)
                .index(1),
        )
        .get_matches();

    let socket_path = matches
        .value_of("SOCKET")
        .expect("Must provide a valid path to the socket used by fand");

    let stream =
        BufReader::new(UnixStream::connect(socket_path).expect("Failed to connect to socket"));

    for line in stream.lines() {
        let line = line.unwrap();
        let mut parts = line.split(":");
        let id = parts.next().unwrap().parse::<usize>().unwrap();
        let operation_name = parts.next().unwrap().trim();
        if id == 7 && operation_name == ">" {
            let the_rest = parts.next().unwrap().parse::<f64>().unwrap();
            println!("{:2.0}", the_rest);
            break;
        }
    }
}
