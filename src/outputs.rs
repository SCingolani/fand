use ::std::{thread, time};
use log::debug;
use rppal::pwm;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// The output that is being controlled. They implement the `Pushable` trait, meaning that they
/// define a way to send (i.e. push) a value to the output.
#[derive(Serialize, Deserialize)]
pub enum Output {
    PWM,
    External(String),
}

pub trait Pushable {
    fn push(&mut self, val: f64);
}

/// Start the control loop with no exit condition. This takes essentially any iterator which
/// produces `f64`s, which is sampled at a given `rate`, and these values are then fed into the
/// output. NOTE: The current implementation *will not push new values unless the differ by more
/// than 0.001*. This is, of course, very arbitrary and has to change in future versions, possibly
/// providing an adjustable threshold.
pub fn sample_forever(
    mut source: Box<dyn Iterator<Item = f64>>,
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

/// Wrapper around (rppal)[https://crates.io/crates/rppal]'s pwm; it has a fixed frequency in
/// current implementation (see (new)[#method.new]) and also has a special logic if it is starting
/// from a duty cycle of 0: it turns the output to 100% and blocks for 500ms, and then returns to
/// normal operation.
pub struct PWM {
    pin: pwm::Pwm,
    last_zero: bool,
}

impl PWM {
    /// Create a new PWM output; current implementation has fixed 10kHz frequency and inverse
    /// polarity.
    pub fn new() -> Result<PWM, rppal::pwm::Error> {
        // TODO: Remove defaults values here and instead have them as parameters of the Output
        // enum.
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
        // TODO: Code below is specific to my current setup; the special behaviour when starting
        // from 0 could be implemented in an operation, such logic doesn't belong here.
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
        let _ = Command::new(cmd)
            .arg(format!("{}", val))
            .output()
            .expect("External output command failed");
    }
}
