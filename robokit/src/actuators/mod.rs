pub mod axis;
pub mod led;
pub mod spindle;

use crate::error::Error;
use core::task::Poll;

// receive inspired by https://github.com/rtic-rs/rfcs/pull/0052
// poll inspired by https://docs.rs/stepper
pub trait Actuator {
    type Action;
    type Error: Error;

    fn run(&mut self, action: &Self::Action);
    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}
