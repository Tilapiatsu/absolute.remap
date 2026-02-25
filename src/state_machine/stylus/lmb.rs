use crate::state_machine::stylus::{Context, Event, ID, InputEventHelper, Output};
use crate::state_machine::stylus::{idle::Idle, mmb::MMB, rmb::RMB};
use crate::state_machine::{State, Transition};
use evdev::{InputEvent, KeyCode};
use log::info;

pub struct LMB;

impl State<Context, Event, Output> for LMB {
    fn enter(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("lmb pressed");
        vec![self.id().pressed]
    }

    fn exit(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("lmb released");
        vec![self.id().released]
    }

    fn update(&mut self, ctx: &mut Context, event: Event) -> Transition<Context, Event, Output> {
        if !ctx.pen {
            return Transition::Change(Box::new(Idle), vec![self.id().released]);
        }

        let key = KeyCode::new(event.code());
        // let value = event.pressed();

        match key {
            KeyCode::BTN_STYLUS if ctx.touch => {
                ctx.stylus1 = event.pressed();
                Transition::Change(Box::new(RMB), Vec::new())
            }
            KeyCode::BTN_STYLUS2 if ctx.touch => {
                ctx.stylus2 = event.pressed();
                Transition::Change(Box::new(MMB), Vec::new())
            }
            KeyCode::BTN_TOUCH if !ctx.touch => {
                ctx.touch = event.pressed();
                Transition::Change(Box::new(Idle), Vec::new())
            }
            _ => Transition::Stay(Vec::new()),
        }
    }
}

impl LMB {
    pub fn id(&self) -> ID {
        ID {
            keycode: KeyCode::BTN_LEFT,
            pressed: InputEvent::new(1, KeyCode::BTN_LEFT.code(), 1),
            released: InputEvent::new(1, KeyCode::BTN_LEFT.code(), 0),
        }
    }
}
