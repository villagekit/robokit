use crate::util::ref_mut::RefMut;

pub trait Actuator {
    type Action;
    type Output: Activity;

    fn act(self, action: &Self::Action) -> Self::Output;
}

#[derive(Debug, Eq, PartialEq)]
pub enum ActivityError {}

pub trait Activity {
    fn poll(self) -> core::task::Poll<Result<(), ActivityError>>;
}

pub trait Listen<Event> {
    fn listen(&mut self, event: Event);
}

// ref muts

impl<'r, T> Actuator for RefMut<'r, T>
where
    T: Actuator,
{
    type Action = T::Action;
    type Output = T::Output;

    fn act(&mut self, action: &Self::Action) -> Self::Output {
        self.0.act(self, action)
    }
}

impl<'r, T> Activity for RefMut<'r, T>
where
    T: Activity,
{
    fn poll(&mut self) -> core::task::Poll<Result<(), ActivityError>> {
        self.0.poll()
    }
}
