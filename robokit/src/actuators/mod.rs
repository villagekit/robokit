pub mod axis;
pub mod led;
pub mod spindle;

use core::fmt::Debug;
use core::marker::PhantomData;
use core::task::Poll;
use defmt::Format;

use crate::error::Error;

// receive inspired by https://github.com/rtic-rs/rfcs/pull/0052
// poll inspired by https://docs.rs/stepper
pub trait Actuator {
    type Action: Debug + Format;
    type Error: Error;

    fn run(&mut self, action: &Self::Action);
    fn poll(&mut self) -> Poll<Result<(), Self::Error>>;
}

pub trait ActuatorSet {
    type Action: Debug + Format;
    type Id: Copy + Debug + Format;
    type Error: Error;

    fn run(&mut self, id: &Self::Id, action: &Self::Action);
    fn poll(&mut self, id: &Self::Id) -> Poll<Result<(), Self::Error>>;
}

pub struct EmptyActuatorSet<Action> {
    action: PhantomData<Action>,
}

impl<Action> EmptyActuatorSet<Action>
where
    Action: Debug + Format,
{
    pub fn new() -> Self {
        Self {
            action: PhantomData::<Action>,
        }
    }
}

impl<Action> ActuatorSet for EmptyActuatorSet<Action>
where
    Action: Debug + Format,
{
    type Action = Action;
    type Id = ();
    type Error = ();

    fn run(&mut self, _id: &Self::Id, _action: &Self::Action) {}
    fn poll(&mut self, _id: &Self::Id) -> Poll<Result<(), Self::Error>> {
        unreachable!("EmptyActuatorSet::poll is unreachable.")
    }
}

#[macro_export]
macro_rules! actuator_set {
    (
        $type:ident { $($actuator:ident),* },
        $action:ty,
        $id:ident,
        $set:ident,
        $error:ident
    ) => {
        $crate::paste! {
            #[derive(Copy, Clone, Debug, defmt::Format)]
            pub enum $id {
                $(
                    [<$actuator:camel>],
                )*
            }

            #[derive(Copy, Clone, Debug)]
            pub enum $error<
                $(
                    [<$actuator:camel $type:camel>]: core::fmt::Debug,
                )*
            >{
                $(
                    [<$actuator:camel $type:camel>]([<$actuator:camel $type:camel>]),
                )*
            }

            pub struct $set<
                $(
                    [<$actuator:camel $type:camel>],
                )*
            >
            where
                $(
                    [<$actuator:camel $type:camel>]: $crate::actuators::Actuator<Action = $action>,
                )*
            {

                $(
                    [<$actuator:snake $type:snake>]: [<$actuator:camel $type:camel>],
                )*
            }

            impl<
                $(
                    [<$actuator:camel $type:camel>],
                )*
            > $set<
                $(
                    [<$actuator:camel $type:camel>],
                )*
            >
            where
                $(
                    [<$actuator:camel $type:camel>]: $crate::actuators::Actuator<Action = $action>,
                )*
            {
                pub fn new(
                    $(
                        [<$actuator:snake $type:snake>]: [<$actuator:camel $type:camel>],
                    )*
                ) -> Self {
                    Self {
                        $(
                            [<$actuator:snake $type:snake>],
                        )*
                    }
                }
            }

            impl<
                $(
                    [<$actuator:camel $type:camel>],
                )*
            > $crate::actuators::ActuatorSet for $set<
                $(
                    [<$actuator:camel $type:camel>],
                )*
            >
            where
                $(
                    [<$actuator:camel $type:camel>]: $crate::actuators::Actuator<Action = $action>,
                    [<$actuator:camel $type:camel>]::Error: core::fmt::Debug,
                )*
            {
                type Action = $action;
                type Id = $id;
                type Error = $error<
                    $(
                        [<$actuator:camel $type:camel>]::Error,
                    )*
                >;

                fn run(&mut self, id: &Self::Id, action: &Self::Action) {
                    match id {
                        $(
                            $id::[<$actuator:camel>] => {
                                self
                                    .[<$actuator:snake $type:snake>]
                                    .run(action)
                            },
                        )*
                    }
                }

                fn poll(&mut self, id: &Self::Id) -> core::task::Poll<Result<(), Self::Error>> {
                    match id {
                        $(
                            $id::[<$actuator:camel>] => {
                                self
                                    .[<$actuator:snake $type:snake>]
                                    .poll()
                                    .map_err($error::[<$actuator:camel $type:camel>])
                            },
                        )*
                    }
                }
            }
        }
    };
}
