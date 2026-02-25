use crate::state_machine::stylus::{Context, Event, ID, InputEventHelper, Output};
use crate::state_machine::stylus::{idle::Idle, lmb::LMB};
use crate::state_machine::{State, Transition};
use evdev::{InputEvent, KeyCode};
use log::info;

pub struct MMB;

impl State<Context, Event, Output> for MMB {
    fn enter(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("mmb pressed");
        vec![self.id().pressed]
    }

    fn exit(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("mmb released");
        vec![self.id().released]
    }

    fn update(&mut self, ctx: &mut Context, event: Event) -> Transition<Context, Event, Output> {
        if !ctx.pen {
            return Transition::Change(Box::new(Idle), vec![self.id().released]);
        }

        let key = KeyCode::new(event.code());
        // let value = event.pressed();

        match key {
            KeyCode::BTN_STYLUS2 if ctx.touch => {
                ctx.stylus2 = event.pressed();
                Transition::Change(Box::new(LMB), Vec::new())
            }
            KeyCode::BTN_TOUCH if !ctx.touch => {
                ctx.touch = event.pressed();
                Transition::Change(Box::new(Idle), Vec::new())
            }
            _ => Transition::Stay(Vec::new()),
        }
    }
}

impl MMB {
    pub fn id(&self) -> ID {
        ID {
            keycode: KeyCode::BTN_MIDDLE,
            pressed: InputEvent::new(1, KeyCode::BTN_MIDDLE.code(), 1),
            released: InputEvent::new(1, KeyCode::BTN_MIDDLE.code(), 0),
        }
    }
}
