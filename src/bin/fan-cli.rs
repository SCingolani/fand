use clap::{App, Arg};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

fn main() {
    // Parse command line parameters:
    let matches = App::new("Fand CLI interface")
        .version("0.1")
        .author("")
        .about("Command line client to retrieve internal state of the fand control loop")
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
        println!("{}", line);
        let mut parts = line.split(":");
        let id = parts.next().unwrap().parse::<usize>();
        let operation_name = parts.next().unwrap().trim();
        let the_rest = parts.collect::<Vec<&str>>().join(":");
        let the_rest: serde_json::Result<HashMap<String, f64>> = serde_json::from_str(&the_rest);
        println!("The operation is {} at index {:?}", operation_name, id);
        if let Ok(the_rest) = the_rest {
            match operation_name {
                "PID" => println!(
                    "P: {}\tI: {}\t D: {}\t",
                    the_rest["P"], the_rest["I"], the_rest["D"]
                ),
                _ => println!(""),
            }
        } else {
            println!("Failed to parse the rest");
        }
    }
}
