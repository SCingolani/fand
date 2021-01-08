use std::iter;

use pifan::inputs::Input;
use pifan::operations::parameters::*;

use pifan::outputs::{sample_forever, PWM};

use simplelog::*;

use pid::Pid;

fn main() {
    println!("Hello, world!");

    TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Mixed).unwrap();

    let dummy_data = vec![1.0, 2.0, 3.0];
    let operation = IdentityParameters;
    let operated = operation.apply(dummy_data.into_iter(), None);
    println!("{:?}", operated.collect::<Vec<f64>>());
    let dummy_data = vec![1.0, 2.0, 3.0];
    let operation = PIDParameters {
        pid: Pid::new(1.0, 0.0, 0.0, 100.0, 100.0, 100.0, 0.0),
        offset: 0,
    };
    let operated = operation.apply(dummy_data.into_iter(), None);
    println!("{:?}", operated.collect::<Vec<f64>>());

    let dummy_data = iter::repeat(1.0);
    let operation = DampenedOscillatorParameters {
        m: 10.0,
        k: 1.0,
        dt: 0.5,
        target: 0.0,
    };
    let operated = operation.apply(dummy_data, None).step_by(2).take(20);
    println!("{:?}", operated.collect::<Vec<f64>>());
    /*
      const min: i64 = 0;
      const step: i64 = 5;
      const max: i64 = 100;
      let mut val: i64 = min;
      let mut dir: bool = true;
      let output = PWM::new().unwrap();



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

    let input = Input::RPiCpuTemp;

    let output = PWM::new().unwrap();

    let average = AverageParameters { n: 5 };

    let pid = PIDParameters {
        pid: Pid::new(2., 2.0, 5., 100., 10., 30., 35.),
        offset: 30,
    };
    let supersampler = SupersampleParameters { n: 100 };
    let clipper = ClipParameters {
        min: 30.0,
        max: 100.0,
    };
    let clipper2 = ClipParameters {
        min: 30.0,
        max: 100.0,
    };
    let dampener = DampenedOscillatorParameters {
        m: 0.5,
        k: 2.,
        dt: 0.25,
        target: 0.0,
    };
    let dampener2 = DampenedOscillatorParameters {
        m: 1.0,
        k: 1.,
        dt: 0.25,
        target: 0.0,
    };
    //let operations: Vec<& Operation> = vec![&pid, &clipper, &supersampler, &dampener];
    //
    let  operated = Box::new(clipper2
        .apply(dampener2.apply(
            dampener
                .apply(
                    supersampler.apply(clipper.apply(pid.apply(average.apply(input, None), None), None), None),
                None)
       , None )
                .step_by(4),
         None)/*
        .map(|x| {
            let val = (x * 1000.) as u64;
            if val == 25000 {
                0.
            } else if val < 35000 {
                35.
            } else {
                x
            }
        })*/);

    sample_forever(operated, output, 1000);

    /*
        for i in 1..10 {
            thread::sleep(time::Duration::from_secs(1));
            println!("{:?}", operated.next());
        }
    */
}
