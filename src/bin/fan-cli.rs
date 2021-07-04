use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

use std::collections::HashMap;

fn main() {
    let stream = BufReader::new(UnixStream::connect("/tmp/fand.socket").unwrap());
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
