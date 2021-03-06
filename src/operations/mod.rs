//! This module defines a series of operations that act on iterators. In this sense an operation is
//! an iterator adaptor: it consumes an iterator to produce a new one. The general pattern is
//! inspired by the way iterator adaptors work in the itertools crate, but also using a "builder"
//! kind of approach: For each possible operation there is a struct *`OperationName`*`Parameters`
//! which contain the parameters that define the operation itself. These structs implement a common
//! trait -[Operation][parameters::Operation]- which takes the parameters and an input iterator, to
//! produce the new iterator that applies the corresponding operation. There is also an enum over
//! the different parameter structs to facilitate serialization / deserialization.

use parameters::*;

// export the parameters under the operations module
pub mod parameters;

use serde::Serialize;

use log::debug;
use tracing::{event, Level};

use pid::Pid;
use std::iter::Fuse;

/// The identity operation.
#[derive(Debug, Serialize)]
pub struct Identity<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    #[serde(skip_serializing)]
    monitor: Option<Monitor>,
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
    fn apply(self, iter: I, monitor: Option<Monitor>) -> Identity<I> {
        Identity {
            iter: iter.fuse(),
            monitor,
        }
    }
}

/// A PID control operation.
#[derive(Debug, Serialize)]
pub struct PID<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    pid: Pid<f64>,
    offset: u32,
    #[serde(skip_serializing)]
    monitor: Option<Monitor>,
}

impl<I> Iterator for PID<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
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
                self.monitor.as_ref().and_then(|monitor| {
                    Some(monitor.send(format!(
                        "PID: {{\"P\": {}, \"I\": {}, \"D\": {}}}\n",
                        p, i, d
                    )))
                });
                let sum = (p + i + d) as u32;
                (self.offset + std::cmp::max(0, std::cmp::min(100, sum))) as f64
            };
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!(">:{}\n", output))));
            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(
                Level::TRACE,
                category = "monitoring",
                operation = "PID",
                "{}",
                serialized
            );
            //self.monitor.as_ref().and_then(|monitor| Some(monitor.send(format!("PID: {}\n", serialized))));
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
    fn apply(self, iter: I, monitor: Option<Monitor>) -> PID<I> {
        PID {
            iter: iter.fuse(),
            pid: self.pid,
            offset: self.offset,
            monitor,
        }
    }
}

/// A (critically) dampened oscillator operation.
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
    c: f64, // should not be manually set! but we save it to don't have to calculate sqrt every time step
    pos: f64,
    vel: f64,
    acc: f64,
    #[serde(skip_serializing)]
    monitor: Option<Monitor>,
}

impl<I> Iterator for DampenedOscillator<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
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
            event!(
                Level::TRACE,
                category = "monitoring",
                operation = "DampenedOscillator",
                "{} {}",
                {
                    println!("Evaluated");
                    "hi"
                },
                serialized
            );
            self.monitor.as_ref().and_then(|monitor| {
                Some(monitor.send(format!("DampenedOscillator: {}\n", serialized)))
            });
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!(">:{}\n", new_pos))));

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
    fn apply(self, iter: I, monitor: Option<Monitor>) -> DampenedOscillator<I> {
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
            monitor,
        }
    }
}

/// A clipping operation.
#[derive(Debug, Serialize)]
pub struct Clip<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    max: u64,
    min: u64,
    #[serde(skip_serializing)]
    monitor: Option<Monitor>,
}

impl<I> Iterator for Clip<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
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
            event!(
                Level::TRACE,
                category = "monitoring",
                operation = "Clip",
                "{}",
                serialized
            );
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!("Clip: {}\n", serialized))));
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!(">:{}\n", out))));

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
    fn apply(self, iter: I, monitor: Option<Monitor>) -> Clip<I> {
        Clip {
            iter: iter.fuse(),
            max: (self.max * 1000.) as u64,
            min: (self.min * 1000.) as u64,
            monitor,
        }
    }
}

/// An operation which returns `x` if `x` is at least some value, `0` otherwise.
#[derive(Debug, Serialize)]
pub struct AtLeast<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    val: u64,
    #[serde(skip_serializing)]
    monitor: Option<Monitor>,
}

impl<I> Iterator for AtLeast<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if let Some(val) = self.iter.next() {
            // Clip and ordering not implemented for f64; so we round up. Here we assume we are
            // generally dealing with values between 0 and 100...
            let mut tmp: u64 = (val * 1000.) as u64; // get up to a thoudansth of the value
            if tmp < self.val {
                tmp = 0;
            }

            let out: f64 = (tmp as f64) / 1000.;

            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(
                Level::TRACE,
                category = "monitoring",
                operation = "AtLeast",
                "{}",
                serialized
            );
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!("AtLeast: {}\n", serialized))));
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!(">:{}\n", out))));

            Some(out)
        } else {
            None
        }
    }
}

impl<I> Operation<I, AtLeast<I>> for AtLeastParameters
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I, monitor: Option<Monitor>) -> AtLeast<I> {
        AtLeast {
            iter: iter.fuse(),
            val: (self.val * 1000.) as u64,
            monitor,
        }
    }
}

/// A super-sampling operation.
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
    #[serde(skip_serializing)]
    monitor: Option<Monitor>,
}

impl<I> Iterator for Supersample<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.last_val.is_some() && self.count < self.n {
            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(
                Level::TRACE,
                category = "monitoring",
                operation = "Supersample",
                "{}",
                serialized
            );
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!("Supersample: {}\n", serialized))));
            self.monitor.as_ref().and_then(|monitor| {
                Some(monitor.send(format!(">:{}\n", self.last_val.unwrap_or(-1.0))))
            });
            self.count += 1;
            self.last_val
        } else if let Some(val) = self.iter.next() {
            self.last_val = Some(val);
            self.count = 1;
            let serialized: String = serde_json::to_string(&self).unwrap();
            event!(
                Level::TRACE,
                category = "monitoring",
                operation = "Supersample",
                "{}",
                serialized
            );
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!("Supersample: {}\n", serialized))));
            self.monitor
                .as_ref()
                .and_then(|monitor| Some(monitor.send(format!(">:{}\n", val))));
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
    fn apply(self, iter: I, monitor: Option<Monitor>) -> Supersample<I> {
        Supersample {
            iter: iter.fuse(),
            n: self.n,
            count: 1,
            last_val: None,
            monitor,
        }
    }
}

/// A sub-sampling operation.
#[derive(Debug, Serialize)]
pub struct Subsample<I>
where
    I: Iterator,
{
    #[serde(skip_serializing)]
    iter: Fuse<I>,
    n: usize,
    #[serde(skip_serializing)]
    monitor: Option<Monitor>,
}

impl<I> Iterator for Subsample<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        for _i in 0..self.n {
            let _discard = self.iter.next();
        }
        let next = self.iter.next();
        let serialized: String = serde_json::to_string(&self).unwrap();
        event!(
            Level::TRACE,
            category = "monitoring",
            operation = "Subsample",
            "{}",
            serialized
        );
        self.monitor
            .as_ref()
            .and_then(|monitor| Some(monitor.send(format!("Subsample: {}\n", serialized))));
        self.monitor
            .as_ref()
            .and_then(|monitor| Some(monitor.send(format!(">:{}\n", next.unwrap_or(-1.0)))));
        next
    }
}

impl<I> Operation<I, Subsample<I>> for SubsampleParameters
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I, monitor: Option<Monitor>) -> Subsample<I> {
        Subsample {
            iter: iter.fuse(),
            n: self.n,
            monitor,
        }
    }
}

/// A moving average operation.
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
    #[serde(skip_serializing)]
    monitor: Option<Monitor>,
}

impl<I> Iterator for Average<I>
where
    I: Iterator<Item = f64>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if let Some(val) = self.iter.next() {
            if self.prev_vals.len() < self.n {
                self.prev_vals.push(val);
                let mean = self.prev_vals.iter().sum::<f64>() / (self.prev_vals.len() as f64);
                let serialized: String = serde_json::to_string(&self).unwrap();
                event!(
                    Level::TRACE,
                    category = "monitoring",
                    operation = "Average",
                    "{}",
                    serialized
                );
                self.monitor
                    .as_ref()
                    .and_then(|monitor| Some(monitor.send(format!("Average: {}\n", serialized))));
                self.monitor
                    .as_ref()
                    .and_then(|monitor| Some(monitor.send(format!(">:{}\n", mean))));
                debug!("Average: {:2.4}", mean);
                Some(mean)
            } else {
                self.prev_vals[self.index] = val;
                self.index = (self.index + 1) % self.n;
                let mean = self.prev_vals.iter().sum::<f64>() / (self.prev_vals.len() as f64);
                let serialized: String = serde_json::to_string(&self).unwrap();
                event!(
                    Level::TRACE,
                    category = "monitoring",
                    operation = "Average",
                    "{}",
                    serialized
                );
                self.monitor
                    .as_ref()
                    .and_then(|monitor| Some(monitor.send(format!("Average: {}\n", serialized))));
                self.monitor
                    .as_ref()
                    .and_then(|monitor| Some(monitor.send(format!(">:{}\n", mean))));
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
    fn apply(self, iter: I, monitor: Option<Monitor>) -> Average<I> {
        Average {
            iter: iter.fuse(),
            n: self.n,
            index: 0,
            prev_vals: Vec::new(),
            monitor,
        }
    }
}
