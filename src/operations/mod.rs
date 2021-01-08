//! This module defines a series of operations that act on iterators. In this sense an operation is
//! an iterator adaptor: it consumes an iterator to produce a new one. The general pattern is
//! inspired by the way iterator adaptors work in the itertools crate, but also using a "builder"
//! kind of approach: For each possible operation there is a struct *`OperationName`*`Parameters`
//! which contain the parameters that define the operation itself. These structs implement a common
//! trait -[Operation](trait.Operation)- which takes the parameters and an input iterator, to
//! produce the new iterator that applies the corresponding operation. There is also an enum over
//! the different parameter structs to facilitate serialization / deserialization.

use parameters::*;

// export the parameters under the operations module
pub mod parameters;

use serde::{Serialize};

use log::{debug, trace};
use tracing::{span, event, Level};

use pid::Pid;
use std::iter::Fuse;

#[derive(Debug, Serialize)]
pub struct Identity<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
}

impl<I> Iterator for Identity<I>
where
    I: Iterator,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        self.iter.next()
    }
}

impl<I> Operation<I, Identity<I>> for IdentityParameters
where
    I: Iterator,
{
    fn apply(self, iter: I) -> Identity<I> {
        Identity { iter: iter.fuse() }
    }
}

#[derive(Debug, Serialize)]
pub struct PID<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    pid: Pid<f64>,
    offset: u32,
}

impl<I> Iterator for PID<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        let span = span!(Level::TRACE, "monitoring");
        if let Some(val) = self.iter.next() {
            let control = self.pid.next_control_output(val);
            let output = {
                let p = if control.p.is_sign_negative() {
                    -control.p
                } else {
                    0.
                };
                let i = if control.i.is_sign_negative() {
                    -control.i
                } else {
                    0.
                };
                let d = if control.d.is_sign_negative() {
                    -control.d
                } else {
                    0.
                };
                let sum = (p + i + d) as u32;
                (self.offset + std::cmp::max(0, std::cmp::min(100, sum))) as f64
            };
            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(Level::TRACE, category = "monitoring", operation = "PID", "{}", serialized);
            Some(output)
        } else {
            None
        }
    }
}

impl<I> Operation<I, PID<I>> for PIDParameters
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I) -> PID<I> {
        let serialized: String = serde_json::to_string(&self).unwrap();
        event!(Level::TRACE, category = "monitoring", operation = "DampenedOscillator", "{}", serialized);
        PID {
            iter: iter.fuse(),
            pid: self.pid,
            offset: self.offset,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DampenedOscillator<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    m: f64,
    k: f64,
    dt: f64,
    target: f64,
    c: f64, // should not be manually set! but we save it to don't have to calcualte sqrt every time step
    pos: f64,
    vel: f64,
    acc: f64,
}

impl<I> Iterator for DampenedOscillator<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        let span = span!(Level::TRACE, "monitoring");
        if let Some(val) = self.iter.next() {
            self.target = val;

            let acc = -1.0 * self.k * (self.pos - self.target) - self.c * self.vel;
            let new_pos = self.pos + self.dt * self.vel + 0.5 * self.dt * self.dt * self.acc;
            let fac = self.dt / (2.0 * self.m);
            let new_vel = 1.0 / (1.0 + self.c * fac)
                * (self.vel * (1.0 - self.c * fac) + fac * (self.acc - acc));
            self.acc = acc;
            self.vel = new_vel;
            self.pos = new_pos;

            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(Level::TRACE, category = "monitoring", operation = "DampenedOscillator", "{} {}", {println!("Evaluated"); "hi"} ,serialized);

            Some(new_pos)
        } else {
            None
        }
    }
}

impl<I> Operation<I, DampenedOscillator<I>> for DampenedOscillatorParameters
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I) -> DampenedOscillator<I> {
        let cc = 2_f64 * (self.k * self.m).sqrt();
        DampenedOscillator {
            iter: iter.fuse(),
            m: self.m,
            k: self.k,
            dt: self.dt,
            target: 100.0,
            c: cc,
            pos: 100.0,
            vel: 0.0,
            acc: 0.0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Clip<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    max: u64,
    min: u64,
}

impl<I> Iterator for Clip<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        let span = span!(Level::TRACE, "monitoring");
        if let Some(val) = self.iter.next() {
            // Clip and ordering not implemented for f64; so we round up. Here we assume we are
            // generally dealing with values between 0 and 100...
            let mut tmp: u64 = (val * 1000.) as u64; // get up to a thoudansth of the value
            if tmp > self.max {
                tmp = self.max;
            }
            if tmp < self.min {
                tmp = self.min;
            }

            let out: f64 = (tmp as f64) / 1000.;

            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(Level::TRACE, category = "monitoring", operation = "Clip", "{}", serialized);

            Some(out)
        } else {
            None
        }
    }
}

impl<I> Operation<I, Clip<I>> for ClipParameters
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I) -> Clip<I> {
        Clip {
            iter: iter.fuse(),
            max: (self.max * 1000.) as u64,
            min: (self.min * 1000.) as u64,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Supersample<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    n: usize,
    count: usize,
    last_val: Option<f64>,
}

impl<I> Iterator for Supersample<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        let span = span!(Level::TRACE, "monitoring");
        if self.last_val.is_some() && self.count < self.n {
            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(Level::TRACE, category = "monitoring", operation = "Supersample", "{}", serialized);
            self.count += 1;
            self.last_val
        } else if let Some(val) = self.iter.next() {
            self.last_val = Some(val);
            self.count = 1;
            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(Level::TRACE, category = "monitoring", operation = "Supersample", "{}", serialized);
            Some(val)
        } else {
            None
        }
    }
}

impl<I> Operation<I, Supersample<I>> for SupersampleParameters
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I) -> Supersample<I> {
        Supersample {
            iter: iter.fuse(),
            n: self.n,
            count: 1,
            last_val: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Subsample<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    n: usize,
}

impl<I> Iterator for Subsample<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        let span = span!(Level::TRACE, "monitoring");
        for i in (0..self.n) {
            let _discard = self.iter.next();
        }
        let serialized: String = serde_json::to_string(&self).unwrap();
        event!(Level::TRACE, category = "monitoring", operation = "Subsample", "{}", serialized);
        self.iter.next()
    }
}

impl<I> Operation<I, Subsample<I>> for SubsampleParameters
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I) -> Subsample<I> {
        Subsample {
            iter: iter.fuse(),
            n: self.n,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Average<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    n: usize,
    index: usize,
    prev_vals: Vec<f64>,
}

impl<I> Iterator for Average<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        let span = span!(Level::TRACE, "monitoring");
        if let Some(val) = self.iter.next() {
            if self.prev_vals.len() < self.n {
                self.prev_vals.push(val);
                let mean = self.prev_vals.iter().sum::<f64>() / (self.prev_vals.len() as f64);
                let serialized: String = serde_json::to_string(&self).unwrap();
                event!(Level::TRACE, category = "monitoring", operation = "Average", "{}", serialized);
                debug!("Average: {:2.4}", mean);
                Some(mean)
            } else {
                self.prev_vals[self.index] = val;
                self.index = (self.index + 1) % self.n;
                let mean = self.prev_vals.iter().sum::<f64>() / (self.prev_vals.len() as f64);
                let serialized: String = serde_json::to_string(&self).unwrap();
                event!(Level::TRACE, category = "monitoring", operation = "Average", "{}", serialized);
                debug!("Average: {:2.4}", mean);
                Some(mean)
            }
        } else {
            None
        }
    }
}

impl<I> Operation<I, Average<I>> for AverageParameters
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I) -> Average<I> {
        Average {
            iter: iter.fuse(),
            n: self.n,
            index: 0,
            prev_vals: Vec::new(),
        }
    }
}
