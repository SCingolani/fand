use serde::{Deserialize, Serialize};

use crate::inputs::Input;
use crate::operations::parameters::*;
use crate::outputs::{sample_forever, Output, PWM};

#[derive(Serialize, Deserialize)]
pub struct Pipeline {
    pub input: Input,
    pub operations: Vec<OperationParameters>,
    pub output: Output,
    pub sample_rate: u64,
}

impl Pipeline {
    pub fn start(self) {
        let mut last_iterator: Box<dyn Iterator<Item = f64>> = Box::new(self.input);
        for operation in self.operations {
            // FIXME: the code below defeats the purpose of having the operation trait...
            // need to figure out how to solve this... eventually some match like below will
            // show up somewhere to deal with the different operations, but at this point here
            // we shouldn't need to match I think...
            last_iterator = match operation {
                OperationParameters::Identity(op) => Box::new(op.apply(last_iterator)),
                OperationParameters::PID(op) => Box::new(op.apply(last_iterator)),
                OperationParameters::DampenedOscillator(op) => Box::new(op.apply(last_iterator)),
                OperationParameters::Clip(op) => Box::new(op.apply(last_iterator)),
                OperationParameters::Supersample(op) => Box::new(op.apply(last_iterator)),
                OperationParameters::Subsample(op) => Box::new(op.apply(last_iterator)),
                OperationParameters::Average(op) => Box::new(op.apply(last_iterator)),
            }
        }
        // TODO: Below code should be generalized if more outputs are to be implemented; is here a
        // good point to call the constructors? How to generalize over different types? How to deal
        // with errors?
        let output = match self.output {
            Output::PWM => PWM::new().unwrap(),
            Output::Sink => unimplemented!(),
        };

        sample_forever(&mut last_iterator, output, self.sample_rate);
    }
}
