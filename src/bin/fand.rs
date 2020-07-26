use std::{thread, time, iter};

use log::debug;

use pifan::operations::*;
use pifan::inputs::RPiCpuTemp;
use pifan::outputs::{sample_forever, PWM};

use simplelog::*;

use pid::Pid;

fn main() {
    println!("Hello, world!");

    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed
    ).unwrap();

    let dummy_data = vec![1.0,2.0,3.0];
    let operation = IdentityOperation;
    let operated = operation.apply(dummy_data.into_iter());
    println!("{:?}", operated.collect::<Vec<f64>>());
    let dummy_data = vec![1.0,2.0,3.0];
    let operation = PIDOperation{pid: Pid::new(1.0,0.0,0.0,100.0,100.0,100.0, 0.0)};
    let operated = operation.apply(dummy_data.into_iter());
    println!("{:?}", operated.collect::<Vec<f64>>());

    let dummy_data = iter::repeat(1.0);
    let operation = CriticallyDampenerOperation{
        m: 1.0,
        k: 1.0,
        dt: 0.5,
        target: 0.0,
    };
    let operated = operation.apply(dummy_data).step_by(2).take(20);
    println!("{:?}", operated.collect::<Vec<f64>>());

    const min: u64 = 45;
    const step: u64 = 5;
    const max: u64 = 60;
    let mut val: u64 = min;
    let mut dir: bool = true;
    let output = PWM::new().unwrap();

    /*

    loop {
        if dir {
            val += step;
        } else {
            val -= step;
        }
        if val > max {
            val = max;
            dir = !dir;
        } else if val < min {
            val = min;
            dir = !dir;
        }
        debug!("setting duty to {}", (val as f64) / 100.);
        output.pin.set_duty_cycle((val as f64) / 100.);

        thread::sleep(time::Duration::from_secs(5));
    }
    */


    let input = RPiCpuTemp;

    let output = PWM::new().unwrap();

    let pid = PIDOperation{pid: Pid::new(5.,1.0,1.,100.,45.,25.,36.5)};
    let supersampler = SupersampleOperation {
        n: 10,
    };
    let clipper = ClipOperation {
        min: 45.0,
        max: 100.0,
    };
    let dampener = CriticallyDampenerOperation{
        m: 1.0,
        k: 1.0,
        dt: 0.5,
        target: 0.0,
    };
    //let operations: Vec<& Operation> = vec![&pid, &clipper, &supersampler, &dampener];
    let mut operated = dampener.apply(
        supersampler.apply(
            clipper.apply(
                pid.apply(input.into_iter())
            )
        )
        ).step_by(2);

    sample_forever(&mut operated, output, 1000);

/*
    for i in 1..10 {
        thread::sleep(time::Duration::from_secs(1));
        println!("{:?}", operated.next());
    }
*/
}
