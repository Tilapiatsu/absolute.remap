use crate::state_machine::stylus::{Context, Event, ID, InputEventHelper, Output};
use crate::state_machine::stylus::{idle::Idle, lmb::LMB};
use crate::state_machine::{State, Transition};
use evdev::{InputEvent, KeyCode};
use log::info;

pub struct RMB;

impl State<Context, Event, Output> for RMB {
    fn enter(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("rmb pressed");
        vec![self.id().pressed]
    }

    fn exit(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("rmb released");
        vec![self.id().released]
    }

    fn update(&mut self, ctx: &mut Context, event: Event) -> Transition<Context, Event, Output> {
        if !ctx.pen {
            return Transition::Change(Box::new(Idle), vec![self.id().released]);
        }

        let key = KeyCode::new(event.code());

        match key {
            KeyCode::BTN_STYLUS if ctx.touch => Transition::Change(Box::new(LMB), Vec::new()),
            KeyCode::BTN_TOUCH if !ctx.touch => Transition::Change(Box::new(Idle), Vec::new()),
            _ => Transition::Stay(Vec::new()),
        }
    }
}

impl RMB {
    pub fn id(&self) -> ID {
        ID {
            keycode: KeyCode::BTN_RIGHT,
            pressed: InputEvent::new(1, KeyCode::BTN_RIGHT.code(), 1),
            released: InputEvent::new(1, KeyCode::BTN_RIGHT.code(), 0),
        }
    }
}
