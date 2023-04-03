pub mod axis;
pub mod led;
pub mod spindle;

use core::fmt::Debug;
use core::task::Poll;

// receive inspired by https://github.com/rtic-rs/rfcs/pull/0052
// poll inspired by https://docs.rs/stepper
pub trait Actuator<Action> {
    type Error: Debug;

    fn receive(&mut self, action: &Action);
    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}
