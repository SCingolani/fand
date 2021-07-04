use std::vec;

use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;
use std::sync::Mutex;

use log::{debug, trace};

use pifan::inputs::Input;
use pifan::operations::parameters::*;
use pifan::outputs::Output;
use pifan::pipeline::Pipeline;

use pid::Pid;

use std::fs::File;
use std::os::unix::fs::PermissionsExt;

use simplelog::*;

use clap::{App, Arg};

fn bind_socket_and_listen(socket_path: &str, pipeline: Pipeline) {
    let listener = {
        debug!("Starting UNIX socket at: {}", socket_path);
        let listener = UnixListener::bind(socket_path)
            .expect(format!("Failed to open socket at {}", socket_path).as_str());
        // TODO: Hack to make it easy to use the socket; setting such permissions doesn't feel
        // very UNIX-y
        std::fs::metadata(socket_path)
            .map(|metadata| metadata.permissions())
            .map(|mut perms| {
                perms.set_mode(0o666);
                perms
            }) // read write for user and group and everybody
            .and_then(|perms| std::fs::set_permissions(socket_path, perms))
            .expect("Failed to set permissions on socket");
        listener
    };

    let clients: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));

    let rx = pipeline.start(true).unwrap();

    let clients_copy = Arc::clone(&clients);

    std::thread::spawn(move || {
        while let Ok(val) = rx.recv() {
            let current_clients = &mut *clients_copy.lock().unwrap();
            let mut to_del: Vec<usize> = Vec::new();
            for (iclient, mut client) in current_clients.iter().enumerate() {
                let res = client.write_all(val.as_bytes());
                if res.is_err() {
                    debug!("Error while writing data to client; will forget client. Client: {:?}. Err: {:?}", client, res);
                    to_del.push(iclient);
                }
            }

            // inefficient but should be OK since adding and removing clients should be rare
            if !to_del.is_empty() {
                let mut i: usize = 0;
                current_clients.retain(|_| (!to_del.contains(&i), i += 1).0); // this doesn't look idiomatic, but it was taken from the examples given in the std documentation...
            }
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut current_clients = clients.lock().unwrap();
                current_clients.push(stream);
            }
            Err(err) => {
                debug!("Error while handling incoming connection");
                break;
            }
        }
    }
}

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
        .arg(
            Arg::with_name("socket")
                .short("s")
                .long("socket")
                .value_name("SOCKET_PATH")
                .help(
                    "Use a unix socket at SOCKET_PATH to broadcast internal state of control loop",
                )
                .takes_value(true),
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

    // If a UNIX socket is requested we need to fork to serve clients and to perform the control
    // loop, otherwise we just execute the control loop in the main thread.

    match matches.value_of("socket") {
        Some(socket_path) => bind_socket_and_listen(socket_path, pipeline),
        None => {
            pipeline.start(false);
        } // in current implementation this is blocking and will never return
    };

    debug!("Something went wrong 😅");

    unreachable!();
}
