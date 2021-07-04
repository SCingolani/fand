use serde::{Deserialize, Serialize};

use std::fs;
use std::process::Command;

use log::debug;

#[derive(Serialize, Deserialize)]
pub enum Input {
    RPiCpuTemp,
    External(String),
}

impl Iterator for Input {
    type Item = f64;

    #[inline]
    fn next(&mut self) -> Option<f64> {
        match self {
            Input::RPiCpuTemp => {
                let file_content = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").ok();
                let the_temp = file_content
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .map(|x| x / 1_000_f64);
                debug!("Temperature is: {:2.2?}", the_temp);
                the_temp
            }
            Input::External(cmd) => {
                // TODO: rudimentary implementation for testing purposes
                let command_output = Command::new(cmd).output().expect("External input command failed");
                let output_string = String::from_utf8(command_output.stdout).expect("Failed to parse external input as string");
                Some(output_string.trim().parse::<f64>().expect("Failed to parse external input as float"))
            }
        }
    }
}
