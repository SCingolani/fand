use serde::{Deserialize, Serialize};

use std::fs;

use log::debug;

#[derive(Serialize, Deserialize)]
pub enum Input {
    RPiCpuTemp,
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
        }
    }
}
