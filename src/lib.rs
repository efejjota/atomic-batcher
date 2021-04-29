#![cfg_attr(test, deny(warnings))]

//! ## Example
//! ```rust
//! extern crate atomic_batcher;
//! extern crate tokio;

//! use atomic_batcher::*;
//! use std::time::{Duration, Instant};
//! use tokio::prelude::*;
//! use tokio::timer::Delay;

//! fn main() {
//!   let when = Instant::now() + Duration::from_millis(2000);
//!   let run = |val: Vec<u64>, _batcher: &mut Batcher<u64>| -> () {
//!     println!("{:?}", val);  
//!   };
//!
//!   // Create a batcher with a run function which will be called  
//!   // when batcher's inner state `running` is OFF and inner state `pending_batch`
//!   // is not empty.
//!   let mut batcher = Batcher::new(run);
//!
//!   // Before this first append, batcher's inner state `running` is initial OFF,
//!   // so batcher will call the run function with the append value directly,
//!   // then inner state `running` is ON.
//!   batcher.append(vec![1, 2, 3]);
//!
//!   // Now because inner state `running` is ON, run function won't be called.
//!   // But the data `vec![4, 5, 6]` and `vec![7, 8, 9]` will be pushed to
//!   // batcher's `pending_batch`.
//!   batcher.append(vec![4, 5, 6]);
//!   batcher.append(vec![7, 8, 9]);
//!
//!   // Now `pending_batch` is vec![4, 5, 6, 7, 8, 9].
//!   // After 2 seconds, batcher.done get called which will turn `running` to OFF,
//!   // then call run function with `pending_batch`.
//!   // Finally turn `running` to ON again.
//!   let task = Delay::new(when)
//!   .and_then(move |_| {
//!     batcher.done(Ok(()));
//!     Ok(())
//!   })
//!   .map_err(|e| panic!("delay errored; err={:?}", e));
//!   tokio::run(task);
//! }
//! ```
//! Running the above example will print
//! ```sh
//! [1, 2, 3]
//!
//! // two seconds later
//! [4, 5, 6, 7, 8, 9]
//! ```

/// Batching representation.
pub struct Batcher<T> {
  running: bool,
  pending_batch: Vec<T>,
  pending_callbacks: Vec<fn(Result<(), &str>) -> ()>,
  callbacks: Vec<fn(Result<(), &str>) -> ()>,
  run: fn(Vec<T>, &mut Batcher<T>) -> (),
}

impl<T> Batcher<T> {
  /// Create a new batcher with a run function.
  pub fn new(run: fn(Vec<T>, &mut Batcher<T>) -> ()) -> Self {
    Batcher {
      running: false,
      pending_batch: Vec::new(),
      pending_callbacks: Vec::new(),
      callbacks: Vec::new(),
      run,
    }
  }
  /// Accept an array of values and a callback.
  /// The accepted callback is called when the batch containing the values have been run.
  pub fn append(&mut self, val: Vec<T>) -> () {
    self.appendcb(val, |_|{})
  }

  pub fn appendcb(&mut self, val: Vec<T>, cb: fn(Result<(), &str>) -> ()) -> () {
    if self.running {
      if self.pending_batch.len() == 0 {
        self.pending_callbacks = Vec::new();
      }
      self.pending_batch.extend(val);
      self.callbacks.push(cb);
    } else {
      self.callbacks = vec![cb];
      self.running = true;
      (self.run)(val, self);
    }
  }
  /// Turn batcher's running state to off. then call the run function.
  pub fn done(&mut self, err: Result<(), &str>) -> () {
    for cb in self.callbacks.iter() {
      cb(err)
    }
    self.running = false;
    self.callbacks = self.pending_callbacks.drain(..).collect();
    let nextbatch: Vec<T> = self.pending_batch.drain(..).collect();
    if nextbatch.is_empty() && self.callbacks.is_empty() {
      return;
    }
    self.running = true;
    (self.run)(nextbatch, self);
  }
}
