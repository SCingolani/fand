use std::vec;

use std::os::unix::net::{UnixStream, UnixListener};
use std::sync::Mutex;
use std::sync::Arc;
use std::io::Write;

use log::{debug, trace};
use tracing_subscriber;

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
                    OperationParameters::Average(AverageParameters { n: 5 }),
                    OperationParameters::PID(PIDParameters {
                        pid: Pid::new(2., 2.0, 5., 100., 10., 30., 35.),
                        offset: 30,
                    }),
                    OperationParameters::Clip(ClipParameters {
                        min: 30.0,
                        max: 100.0,
                    }),
                    OperationParameters::Supersample(SupersampleParameters { n: 100 }),
                    OperationParameters::DampenedOscillator(DampenedOscillatorParameters {
                        m: 0.5,
                        k: 2.,
                        dt: 0.25,
                        target: 0.0,
                    }),
                    OperationParameters::DampenedOscillator(DampenedOscillatorParameters {
                        m: 1.0,
                        k: 1.,
                        dt: 0.25,
                        target: 0.0,
                    }),
                    OperationParameters::Clip(ClipParameters {
                        min: 30.0,
                        max: 100.0,
                    }),
                    OperationParameters::Subsample(SubsampleParameters { n: 4 }),
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

    const SOCKET_ADDRESS: &str = "/tmp/fand.socket";
    debug!("Starting UNIX socket at: {}", SOCKET_ADDRESS);
    let listener = UnixListener::bind(SOCKET_ADDRESS).expect("Failed to open socket.");

    let clients: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));

    let rx = pipeline.start(true).expect("Didn't receive monitoring channel");

    let clients_copy = Arc::clone(&clients);

    std::thread::spawn(move || {
        for val in rx.iter() {
            let mut current_clients = &mut *clients_copy.lock().unwrap();
            for client in current_clients {
                client.write_all(val.as_bytes()).unwrap();
            }
        }
    }
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut current_clients = clients.lock().unwrap();
                current_clients.push(stream);
            },
            Err(err) => break,
        }
    }

    debug!("Exitting");
}
