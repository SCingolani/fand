use log::{debug, trace};

use pid::Pid;
use std::iter::Fuse;

pub mod parameters;
use parameters::*;

/// An operation is essentially an iterator adaptor; it defines a single
/// operation which takes in a type with the Iterator trait and returns another
/// type which also has the Iterator trait. Simple cases would just map a
/// function over the input iterator to produce a new one; but more complicated
/// ones may do different things. This approach here is taken from the itertool
/// crate which defines a bunch of iterator adaptors but without defining a generic trait over
/// them.
pub trait Operation<I, J>
where
    I: Iterator,
    J: Iterator,
{
    /// Applies the operation on the input iterator to produce a new one.
    fn apply(self, iter: I) -> J;
}

/// The actual implementation of the operation
#[derive(Debug)]
pub struct Identity<I>
where
    I: Iterator,
{
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

impl<I> Operation<I, Identity<I>> for IdentityOperation
where
    I: Iterator,
{
    fn apply(self, iter: I) -> Identity<I> {
        Identity { iter: iter.fuse() }
    }
}

/// The actual implementation of the operation
#[derive(Debug)]
pub struct PID<I>
where
    I: Iterator,
{
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
            trace!(
                "PID: {:2.2} -> {:2.2} {:2.2} {:2.2} -> {:2}({:2.2})",
                val,
                control.p,
                control.i,
                control.d,
                output,
                control.output
            );
            Some(output)
        } else {
            None
        }
    }
}

impl<I> Operation<I, PID<I>> for PIDOperation
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I) -> PID<I> {
        trace!("PID created with {:?}", self.pid);
        PID {
            iter: iter.fuse(),
            pid: self.pid,
            offset: self.offset,
        }
    }
}

/// The actual implementation of the operation
#[derive(Debug)]
pub struct CriticallyDampener<I>
where
    I: Iterator,
{
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

impl<I> Iterator for CriticallyDampener<I>
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

            trace!(
                "CriticallyDampener: x= {}; v= {}; a= {}",
                new_pos,
                new_vel,
                acc
            );
            Some(new_pos)
        } else {
            None
        }
    }
}

impl<I> Operation<I, CriticallyDampener<I>> for CriticallyDampenerOperation
where
    I: Iterator<Item = f64>,
{
    fn apply(self, iter: I) -> CriticallyDampener<I> {
        let cc = 2_f64 * (self.k * self.m).sqrt();
        CriticallyDampener {
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

/// The actual implementation of the operation
#[derive(Debug)]
pub struct Clip<I>
where
    I: Iterator,
{
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

            trace!("Clip: {:2.2} -> {:2.2}", val, out);

            Some(out)
        } else {
            None
        }
    }
}

impl<I> Operation<I, Clip<I>> for ClipOperation
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

/// The actual implementation of the operation
#[derive(Debug)]
pub struct Supersample<I>
where
    I: Iterator,
{
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
        if self.last_val.is_some() && self.count < self.n {
            trace!(
                "Supersample: Repeated {:?} ({:2})",
                self.last_val,
                self.count
            );
            self.count += 1;
            self.last_val
        } else if let Some(val) = self.iter.next() {
            self.last_val = Some(val);
            self.count = 1;
            trace!("Supersample: Sampled {:2.2}", val);
            Some(val)
        } else {
            None
        }
    }
}

impl<I> Operation<I, Supersample<I>> for SupersampleOperation
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

/// The actual implementation of the operation
#[derive(Debug)]
pub struct Average<I>
where
    I: Iterator,
{
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
        if let Some(val) = self.iter.next() {
            if self.prev_vals.len() < self.n {
                self.prev_vals.push(val);
                let mean = self.prev_vals.iter().sum::<f64>() / (self.prev_vals.len() as f64);
                trace!("Average: Filling vec {:?} ({:2.2})", self.prev_vals, mean);
                debug!("Average: {:2.4}", mean);
                Some(mean)
            } else {
                self.prev_vals[self.index] = val;
                self.index = (self.index + 1) % self.n;
                let mean = self.prev_vals.iter().sum::<f64>() / (self.prev_vals.len() as f64);
                trace!("Average: {:?} -> ({:2.2})", self.prev_vals, mean);
                debug!("Average: {:2.4}", mean);
                Some(mean)
            }
        } else {
            None
        }
    }
}

impl<I> Operation<I, Average<I>> for AverageOperation
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
