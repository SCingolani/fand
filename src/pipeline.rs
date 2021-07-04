use crate::inputs::Input;
use crate::operations::parameters::*;
use crate::outputs::{sample_forever, External, Output, PWM};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

/// A pipeline is nothing more than a runtime-defined series of iterator transformers. That is,
/// starting from an [Input] (an iterator), it creates on the heap a series of
/// [operations][crate::operations::parameters::Operation] which behave as iterator transformers: they consume an
/// iterator on construction, and produce a new one which is the result of applying some
/// transformation to the values of its input iterator. A previous version of this code did not
/// produce this on the heap but rather using the same approach as in the iterator transformers of
/// the standard library, but in that way the construction of the pipeline at runtime (e.g. from a
/// config file) is prevented.
#[derive(Serialize, Deserialize)]
pub struct Pipeline {
    pub input: Input,
    pub operations: Vec<OperationParameters>,
    pub output: Output,
    pub sample_rate: u64,
}

impl Pipeline {
    /// Given a Pipeline, it starts a control loop that polls its input and pushes the processed
    /// values to the output (see (`sample_forever`)[sample_forever]). Current
    /// implementation is quite unintuitive: if no monitoring (`monitored == false`), it blocks
    /// forever by calling `sample_forever` on the current thread; but if monitoring is requested,
    /// it starts a new thread to execute the control loop and returns a channel to access internal
    /// state of the control loop.
    pub fn start(self, monitored: bool) -> Option<mpsc::Receiver<String>> {
        let sample_rate = self.sample_rate;
        let mut last_iterator: Box<dyn Iterator<Item = f64> + Send> = Box::new(self.input);
        let (tx, rx) = mpsc::channel();
        for (index, operation) in self.operations.iter().enumerate() {
            let local_tx = if monitored {
                Some(Monitor {
                    id: index,
                    tx: tx.clone(),
                })
            } else {
                None
            };
            // FIXME: the code below defeats the purpose of having the operation trait...
            // need to figure out how to solve this... eventually some match like below will
            // show up somewhere to deal with the different operations, but at this point here
            // we shouldn't need to match I think...
            // TODO: The above can be fixed by implementing the same approach as in
            // (i3status-rust)[https://github.com/greshake/i3status-rust] for blocks.
            // A match would still be required but things can be simplified by a simple macro like
            // they do on their codebase. It would also enable to have common config or fields
            // across operations (such as the monitor!)
            last_iterator = match operation {
                OperationParameters::Identity(op) => Box::new(op.apply(last_iterator, local_tx)),
                OperationParameters::PID(op) => Box::new(op.apply(last_iterator, local_tx)),
                OperationParameters::DampenedOscillator(op) => {
                    Box::new(op.apply(last_iterator, local_tx))
                }
                OperationParameters::Clip(op) => Box::new(op.apply(last_iterator, local_tx)),
                OperationParameters::AtLeast(op) => Box::new(op.apply(last_iterator, local_tx)),
                OperationParameters::Supersample(op) => Box::new(op.apply(last_iterator, local_tx)),
                OperationParameters::Subsample(op) => Box::new(op.apply(last_iterator, local_tx)),
                OperationParameters::Average(op) => Box::new(op.apply(last_iterator, local_tx)),
            }
        }
        // TODO: Below code should be generalized if more outputs are to be implemented; is here a
        // good point to call the constructors? How to generalize over different types? How to deal
        // with errors?
        let output: Box<dyn crate::outputs::Pushable + Send> = match self.output {
            Output::PWM => Box::new(PWM::new().unwrap()),
            Output::External(cmd) => Box::new(External { cmd }),
        };

        // If running in monitored mode, spawn a new thread, otherwise run pipeline in current
        // thread.
        // TODO: This behaviour is quite unexpected, best solution would be to have two functions,
        // one which spawns a new thread (regardless of monitoring) and another which doesn't.
        if monitored {
            std::thread::spawn(move || sample_forever(last_iterator, output, sample_rate));
            Some(rx)
        } else {
            sample_forever(last_iterator, output, sample_rate);
            None
        }
    }
}
