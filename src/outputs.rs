use serde::{Serialize,Deserialize};

use log::debug;

use ::std::{thread, time};

use rppal::gpio::Gpio;
use rppal::pwm;

#[derive(Serialize,Deserialize)]
pub enum Output {
    PWM,
    Sink,
}


pub trait Pushable {
    fn push(&mut self, val: f64);
}

pub fn sample_forever<I>(source: &mut Iterator<Item = f64>, mut output: I, rate: u64) -> ()
where
    I: Pushable,
{
    loop {
        let next;
        match source.next() {
            Some(val) => next = val,
            None => break,
        };
        let x = next;
        output.push(x);
        thread::sleep(time::Duration::from_millis(rate));
    }
}

pub struct PWM { 
    pin: pwm::Pwm,
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
        Ok(PWM { pin: pwm })
    }
}
impl Pushable for PWM {
    fn push(&mut self, val: f64) {
        debug!("PWM output set to {:2.4}", val / 100_f64);
        self.pin.set_duty_cycle(val / 100_f64);
    }
}
