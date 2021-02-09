use serde::{Deserialize, Serialize};

use crate::inputs::Input;
use crate::operations::parameters::*;
use crate::outputs::{sample_forever, Output, PWM};

use std::sync::mpsc;

#[derive(Serialize, Deserialize)]
pub struct Pipeline {
    pub input: Input,
    pub operations: Vec<OperationParameters>,
    pub output: Output,
    pub sample_rate: u64,
}

impl Pipeline {
    pub fn start(self, monitored: bool) -> Option<mpsc::Receiver<String>> {
        let sample_rate = self.sample_rate;
        let mut last_iterator: Box<dyn Iterator<Item = f64> + Send> = Box::new(self.input);
        let (tx, rx) = mpsc::channel();
        for (index, operation) in self.operations.iter().enumerate() {
            let local_tx = if monitored {
                Some(Monitor{ id: index, tx: tx.clone() })
            } else {
                None
            };
            // FIXME: the code below defeats the purpose of having the operation trait...
            // need to figure out how to solve this... eventually some match like below will
            // show up somewhere to deal with the different operations, but at this point here
            // we shouldn't need to match I think...
            last_iterator = match operation {
                OperationParameters::Identity(op) => Box::new(op.apply(last_iterator, local_tx)),
                OperationParameters::PID(op) => Box::new(op.apply(last_iterator, local_tx)),
                OperationParameters::DampenedOscillator(op) => Box::new(op.apply(last_iterator, local_tx)),
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
        let output = match self.output {
            Output::PWM => PWM::new().unwrap(),
            Output::Sink => unimplemented!(),
        };

        std::thread::spawn(move || sample_forever(last_iterator, output, sample_rate));
        if monitored {
            Some(rx)
        } else {
            None
        }
    }
}
