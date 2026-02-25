pub mod stylus;

pub enum Transition<C, E, O> {
    Stay(Vec<O>),
    Change(Box<dyn State<C, E, O>>, Vec<O>),
}

pub trait State<C, E, O> {
    fn enter(&mut self, _ctx: &mut C) -> Vec<O> {
        Vec::new()
    }

    fn exit(&mut self, _ctx: &mut C) -> Vec<O> {
        Vec::new()
    }

    fn update(&mut self, ctx: &mut C, event: E) -> Transition<C, E, O>;
}

pub struct StateMachine<C, E, O> {
    state: Box<dyn State<C, E, O>>,
}

impl<C, E, O> StateMachine<C, E, O> {
    pub fn new(initial: Box<dyn State<C, E, O>>) -> Self {
        Self { state: initial }
    }

    pub fn handle_event(&mut self, ctx: &mut C, event: E) -> Vec<O> {
        match self.state.update(ctx, event) {
            Transition::Stay(outputs) => outputs,

            Transition::Change(mut new_state, mut outputs) => {
                // collect exit outputs
                outputs.extend(self.state.exit(ctx));

                // collect enter outputs
                outputs.extend(new_state.enter(ctx));

                self.state = new_state;
                outputs
            }
        }
    }
}
