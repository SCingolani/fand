
use serde::{Serialize,Deserialize};

use pid::Pid;

/// Union type to store the description of some operation; this way we can easily
/// serialize/dynamically create operations.
// TODO Is it possible to create a macro that defines this Union?
#[derive(Serialize,Deserialize)]
pub enum OperationDescription {
    // OperationName: OperationParams
    Identity(IdentityOperation),
    PID(PIDOperation),
    DampenedOscillator(CriticallyDampenerOperation),
    Clip(ClipOperation),
    Supersample(SupersampleOperation),
    Average(AverageOperation),
}

/// An operation which just reproduces the input iterator
#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct IdentityOperation;

/// An operation that implements a PID control
#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct PIDOperation {
    /// PID parameters
    pub pid: Pid<f64>,
    /// Value to add to PID output
    pub offset: u32,
}

/// An operation which uses a critcially dampened oscillator to reach a target value
#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct CriticallyDampenerOperation {
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
#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct ClipOperation {
    /// Minimum value of output
    pub min: f64,
    /// Maximum value of output
    pub max: f64,
}

/// An operation that supersamples its input
#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct SupersampleOperation {
    /// How many times to supersample (i.e. it repeats it's input n times before checking for a new
    /// input)
    pub n: usize,
}

/// An operation that averages its input (running average)
#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct AverageOperation {
    /// How many values to average (i.e. size of window for running average)
    pub n: usize,
}

