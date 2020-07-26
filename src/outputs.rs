
use log::debug;

use::std::{thread,time};

use rppal::gpio::Gpio;
use rppal::pwm;

pub trait Output
{
    fn push(&mut self, val: f64);
}

pub fn sample_forever<I>(source: &mut Iterator<Item = f64>, mut output: I, rate: u64) -> ()
    where I: Output
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
    pub pin: pwm::Pwm // TODO remove pub
}

impl PWM 
{
    pub fn new() -> Result<PWM, rppal::pwm::Error> {
        let pwm = pwm::Pwm::with_frequency(pwm::Channel::Pwm0, 20000.0, 0.5, pwm::Polarity::Normal, true)?;
        Ok( PWM {
            pin: pwm,
        })
    }
}

impl Output for PWM
{
    fn push(&mut self, val: f64)
    {
       debug!("PWM output set to {:2.4}", val / 100_f64);
       self.pin.set_duty_cycle(val / 100_f64);
    }
}
