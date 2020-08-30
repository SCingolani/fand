use std::fs;

use log::debug;

pub struct RPiCpuTemp;

impl Iterator for RPiCpuTemp {
    type Item = f64;

    #[inline]
    fn next(&mut self) -> Option<f64> {
        let file_content = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").ok();
        let the_temp = file_content
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|x| x / 1_000_f64);
        debug!("Temperature is: {:2.2?}", the_temp);
        the_temp
    }
}
