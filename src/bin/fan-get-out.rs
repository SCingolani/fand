use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

fn main() {
    let stream = BufReader::new(UnixStream::connect("/tmp/fand.socket").unwrap());
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
