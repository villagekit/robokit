pub mod axis;
pub mod led;
pub mod spindle;

use core::future::Future;

use alloc::boxed::Box;
use async_trait::async_trait;

use crate::error::{BoxError, Error};

// receive inspired by https://github.com/rtic-rs/rfcs/pull/0052
// poll inspired by https://docs.rs/stepper
pub trait Actuator {
    type Action: Sync;
    type Error: Error;
    type Future: Future<Output = Result<(), Self::Error>>;

    fn run(&mut self, action: &Self::Action) -> Self::Future;
}

pub type BoxActuator<Action> = Box<dyn Actuator<Action = Action, Error = BoxError>>;

pub struct BoxifyActuator<A: Actuator>(A);

impl<A: Actuator> BoxifyActuator<A> {
    pub fn new(actuator: A) -> Self {
        Self(actuator)
    }
}

#[async_trait]
impl<A: Actuator> Actuator for BoxifyActuator<A>
where
    A: Send,
    A::Error: 'static,
{
    type Action = A::Action;
    type Error = BoxError;

    async fn run(&mut self, action: &Self::Action) -> Result<(), Self::Error> {
        self.0
            .run(action)
            .await
            .map_err(|error| (Box::new(error) as Box<dyn Error>).into())
    }
}
