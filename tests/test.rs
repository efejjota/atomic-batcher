extern crate atomic_batcher;
extern crate tokio;

use std::sync::mpsc;
use atomic_batcher::*;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Delay;

#[test]
fn run_once() {
  fn run(val: Vec<u64>, done: mpsc::Sender<()>) -> () {
    assert_eq!(val, vec![1, 2, 3]);
  };
  let mut batcher = Batcher::new(run);
  batcher.append(vec![1, 2, 3]);
}

#[test]
fn run_with_done() {
  let run = |val: Vec<u64>, done: mpsc::Sender<()>| -> () {
    if val == vec![1, 2, 3] {
      //batcher.append(vec![4, 5, 6]);
      done.send(());
    } else {
      assert_eq!(val, vec![4, 5, 6]);
    }
  };
  let mut batcher = Batcher::new(run);
  batcher.append(vec![1, 2, 3]);
}

#[test]
fn run_with_callback() {
  let run = |val: Vec<u64>, done: mpsc::Sender<()>| -> () {
    if val == vec![1, 2, 3] {
      done.send(());
    } else {
      assert_eq!(val, vec![]);
    }
  };
  let mut batcher = Batcher::new(run);
  batcher.appendcb(
    vec![1, 2, 3],
    |err| {
      if let Err(s) = err {
        assert_eq!(s, "some wrong");
      }
    },
  );
}

#[test]
fn run_async() {
  let when = Instant::now() + Duration::from_millis(1000);
  let run = |val: Vec<u64>, done: mpsc::Sender<()>| -> () {
    if val != vec![1, 2, 3] {
      assert_eq!(val, vec![4, 5, 6, 7, 8, 9]);
    }
  };
  let mut batcher = Batcher::new(run);
  batcher.append(vec![1, 2, 3]);
  batcher.append(vec![4, 5, 6]);
  batcher.append(vec![7, 8, 9]);
  
  let task = Delay::new(when)
    .and_then(move |_| {
      //batcher.done(Ok(()));
      Ok(())
    })
    .map_err(|e| panic!("delay errored; err={:?}", e));
  tokio::run(task);
}
