use std::vec;

use log::{debug, trace};

use pifan::inputs::Input;
use pifan::operations::parameters::*;
use pifan::outputs::Output;
use pifan::pipeline::Pipeline;

use pid::Pid;

use std::fs::File;

use simplelog::*;

use clap::{App, Arg};

fn main() {
    let matches = App::new("Fan speed control")
        .version("0.1")
        .author("")
        .about("Configurable control system")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    match matches.occurrences_of("v") {
        0 => TermLogger::init(LevelFilter::Error, Config::default(), TerminalMode::Mixed).unwrap(),
        1 => TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).unwrap(),
        2 => TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Mixed).unwrap(),
        3 | _ => println!("Don't be crazy"),
    }

    debug!("Starting with debug information enabled.");
    trace!("Tracing information enabled.");

    let pipeline: Pipeline = match matches.value_of("config") {
        Some(filename) => {
            debug!("Reading configuration from: {}", filename);
            let config_file = File::open(filename).expect("Failed to read config file");
            serde_json::from_reader(config_file).expect("Failed to parse config file")
        }
        None => {
            debug!("Using default configuration (use verbose level 2 to print it out)");
            let default_pipeline = Pipeline {
                input: Input::RPiCpuTemp,
                operations: vec![
                    OperationDescription::Average(AverageOperation { n: 5 }),
                    OperationDescription::PID(PIDOperation {
                        pid: Pid::new(2., 2.0, 5., 100., 10., 30., 35.),
                        offset: 30,
                    }),
                    OperationDescription::Clip(ClipOperation {
                        min: 30.0,
                        max: 100.0,
                    }),
                    OperationDescription::Supersample(SupersampleOperation { n: 100 }),
                    OperationDescription::DampenedOscillator(CriticallyDampenerOperation {
                        m: 0.5,
                        k: 2.,
                        dt: 0.25,
                        target: 0.0,
                    }),
                    OperationDescription::DampenedOscillator(CriticallyDampenerOperation {
                        m: 1.0,
                        k: 1.,
                        dt: 0.25,
                        target: 0.0,
                    }),
                    OperationDescription::Clip(ClipOperation {
                        min: 30.0,
                        max: 100.0,
                    }),
                ],
                output: Output::PWM,
                sample_rate: 1000,
            };
            trace!(
                "{}",
                serde_json::to_string_pretty(&default_pipeline).unwrap()
            );
            default_pipeline
        }
    };

    pipeline.start();

    debug!("Exitting");
}
