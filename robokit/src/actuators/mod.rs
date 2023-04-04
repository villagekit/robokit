pub mod axis;
pub mod led;
pub mod spindle;

use alloc::boxed::Box;

use crate::error::{BoxError, Error};
use core::task::Poll;

// receive inspired by https://github.com/rtic-rs/rfcs/pull/0052
// poll inspired by https://docs.rs/stepper
pub trait Actuator {
    type Action;
    type Error: Error;

    fn run(&mut self, action: &Self::Action);
    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}

pub type BoxActuator<Action> = Box<dyn Actuator<Action = Action, Error = BoxError>>;

pub struct BoxifyActuator<A: Actuator>(A);

impl<A: Actuator> BoxifyActuator<A> {
    pub fn new(actuator: A) -> Self {
        Self(actuator)
    }
}

impl<A: Actuator> Actuator for BoxifyActuator<A>
where
    A::Error: 'static,
{
    type Action = A::Action;
    type Error = BoxError;

    fn run(&mut self, action: &Self::Action) {
        self.0.run(action)
    }
    fn poll(&mut self) -> Poll<Result<(), Self::Error>> {
        self.0
            .poll()
            .map_err(|error| (Box::new(error) as Box<dyn Error>).into())
    }
}
