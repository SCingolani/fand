//! Module containing all the parameters that define the operations.

use serde::{Deserialize, Serialize};

use pid::Pid;

use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Monitor {
    pub id: usize,
    pub tx: Sender<String>,
}

impl Monitor {
    pub fn send(&self, str: String) {
        self.tx
            .send(format!("{}: {}", self.id, str))
            .expect("Failed to send data to monitor; main thread must have crashed.");
    }
}

/// Common trait that all parameters implement which converts the description of the operation
/// itself into the actual iterator that carries it out.
pub trait Operation<I, J>
where
    I: Iterator,
    J: Iterator,
{
    /// Given self and an input iterator; produce a new iterator that applies the operation
    /// described by `self`.
    fn apply(self, iter: I, monitor: Option<Monitor>) -> J;
}

/// Union type to store the description of some operation; this way we can easily
/// serialize/deserialize operations into a single array.
// TODO Is it possible to create a macro that defines this Union?  Turns out yes! Check out typetag
// crate!
#[derive(Serialize, Deserialize)]
pub enum OperationParameters {
    // OperationName: OperationParams
    Identity(IdentityParameters),
    PID(PIDParameters),
    DampenedOscillator(DampenedOscillatorParameters),
    Clip(ClipParameters),
    AtLeast(AtLeastParameters),
    Supersample(SupersampleParameters),
    Subsample(SubsampleParameters),
    Average(AverageParameters),
}

/// An operation which just reproduces the input iterator (mostly for testing purposes; no real use
/// case)
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct IdentityParameters;

/// An operation that implements a PID control
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PIDParameters {
    /// PID parameters
    pub pid: Pid<f64>,
    /// Value to add to PID output
    pub offset: u32,
}

/// An operation which uses a critcially dampened oscillator to reach a target value
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct DampenedOscillatorParameters {
    /// Mass of particle
    pub m: f64,
    /// Spring constant
    pub k: f64,
    /// Integration time step
    pub dt: f64,
    /// Initial target (not very important)
    pub target: f64,
}

/// An operation that clips all values
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ClipParameters {
    /// Minimum value of output
    pub min: f64,
    /// Maximum value of output
    pub max: f64,
}

/// An operation that clamp values below the reference
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct AtLeastParameters {
    /// Reference value
    pub val: f64,
}

/// An operation that supersamples its input
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SupersampleParameters {
    /// How many times to supersample (i.e. it repeats it's input n times before checking for a new
    /// input)
    pub n: usize,
}

/// An operation that subsamples its input
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SubsampleParameters {
    /// How many times to subsample (i.e. it drops it's input n times before providing a new output
    pub n: usize,
}

/// An operation that averages its input (running average)
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct AverageParameters {
    /// How many values to average (i.e. size of window for running average)
    pub n: usize,
}
