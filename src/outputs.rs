use serde::{Deserialize, Serialize};

use log::debug;

use ::std::{thread, time};
use std::process::Command;

use rppal::pwm;

use std::time::Instant;

#[derive(Serialize, Deserialize)]
pub enum Output {
    PWM,
    External(String),
}

pub trait Pushable {
    fn push(&mut self, val: f64);
}

pub fn sample_forever(
    mut source: Box<Iterator<Item = f64>>,
    mut output: Box<dyn Pushable>,
    rate: u64,
) {
    let mut last: f64 = 0.0;
    loop {
        let next: f64 = match source.next() {
            Some(val) => val,
            None => break,
        };
        if (last * 100.).round() as u64 != (next * 100.).round() as u64 {
            output.push(next);
        }
        last = next;
        thread::sleep(time::Duration::from_millis(rate));
    }
}

pub struct PWM {
    pin: pwm::Pwm,
    last_zero: bool,
}

impl PWM {
    pub fn new() -> Result<PWM, rppal::pwm::Error> {
        let pwm = pwm::Pwm::with_frequency(
            pwm::Channel::Pwm0,
            10000.0,
            0.5,
            pwm::Polarity::Inverse,
            true,
        )?;
        Ok(PWM {
            pin: pwm,
            last_zero: false,
        })
    }
}

impl Pushable for PWM {
    fn push(&mut self, val: f64) {
        const START_POWER: f64 = 100.0;
        debug!("PWM output set to {:2.4}", val / 100_f64);
        if val < 10.0 {
            // val is zero?
            self.last_zero = true;
        } else if self.last_zero {
            self.pin.set_duty_cycle(START_POWER / 100_f64).unwrap();
            thread::sleep(time::Duration::from_millis(500));
            self.last_zero = false;
        } else {
            self.last_zero = false;
        }
        self.pin.set_duty_cycle(val / 100_f64).unwrap();
    }
}

pub struct External {
    pub cmd: String,
}

impl Pushable for External {
    fn push(&mut self, val: f64) {
        // TODO: rudimentary implementation for testing purposes
        let cmd = self.cmd.clone();
        let command_output = Command::new(cmd)
            .arg(format!("{}", val))
            .output()
            .expect("External output command failed");
    }
}
