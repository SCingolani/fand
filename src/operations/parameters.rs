
use pid::Pid;


/// Union type to store the description of some operation; this way we can easily
/// serialize/dynamically create operations.
// TODO Is it possible to create a macro that defines this Union?
pub union OperationDescription {
    // OperationName: OperationParams
    identity: (),
    pid: PIDOperation,
    dampenedOscillator: CriticallyDampenerOperation,
    dlip: ClipOperation,
    supersample: SupersampleOperation,
    average: AverageOperation,
}

/// An operation which just reproduces the input iterator
pub struct IdentityOperation;

/// An operation that implements a PID control
#[derive(Clone, Copy)]
pub struct PIDOperation {
    pub pid: Pid<f64>,
    pub offset: u32,
}

/// An operation which uses a critcially dampened oscillator to reach a target value
#[derive(Clone, Copy)]
pub struct CriticallyDampenerOperation {
    pub m: f64,
    pub k: f64,
    pub dt: f64,
    pub target: f64,
}

/// An operation that clips all values
#[derive(Clone, Copy)]
pub struct ClipOperation {
    pub min: f64,
    pub max: f64,
}

/// An operation that supersamples its input
#[derive(Clone, Copy)]
pub struct SupersampleOperation {
    pub n: usize,
}

/// An operation that supersamples its input
#[derive(Clone, Copy)]
pub struct AverageOperation {
    pub n: usize,
}

