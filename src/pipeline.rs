use serde::{Serialize,Deserialize};

use crate::inputs::Input;
use crate::outputs::{Output, PWM, sample_forever};
use crate::operations::parameters::*;
use crate::operations::Operation;

#[derive(Serialize,Deserialize)]
pub struct Pipeline {
    pub input: Input,
    pub operations: Vec<OperationDescription>,
    pub output: Output,
    pub sample_rate: u64,
}

impl Pipeline {
    pub fn start(self) -> () {
        let mut last_iterator: Box<Iterator<Item = f64>> = Box::new(self.input.into_iter());
        for operation in self.operations {
            last_iterator = 
                // FIXME: the code below defeats the purpose of having the operation trait...
                // need to figure out how to solve this... eventually some match like below will
                // show up somewhere to deal with the different operations, but at this point here
                // we shouldn't need to match I think...
              match operation {
                 OperationDescription::Identity(op) => Box::new(op.apply(last_iterator)),
                 OperationDescription::PID(op) => Box::new(op.apply(last_iterator)),
                 OperationDescription::DampenedOscillator(op) => Box::new(op.apply(last_iterator)),
                 OperationDescription::Clip(op) => Box::new(op.apply(last_iterator)),
                 OperationDescription::Supersample(op) => Box::new(op.apply(last_iterator)),
                 OperationDescription::Average(op) => Box::new(op.apply(last_iterator)),
              }
        }
        // TODO: Below code should be generalized if more outputs are to be implemented; is here a
        // good point to call the constructors? How to generalize over different types? How to deal
        // with errors?
        let output = match self.output {
            PWM => PWM::new().unwrap(),
            Sink => unimplemented!(),
        };

        sample_forever(&mut last_iterator, output, self.sample_rate);

    }
}
