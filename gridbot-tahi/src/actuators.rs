use robokit::{actuator_set, AxisAction, LedAction, SpindleAction};

const TICK_TIMER_HZ: u32 = 1_000_000;

actuator_set!(
    Led { Green, Blue, Red },
    LedAction<TICK_TIMER_HZ>,
    LedId,
    LedSet,
    LedSetError
);

actuator_set!(Axis { X }, AxisAction, AxisId, AxisSet, AxisSetError);

actuator_set!(
    Spindle { Main },
    SpindleAction,
    SpindleId,
    SpindleSet,
    SpindleSetError
);
