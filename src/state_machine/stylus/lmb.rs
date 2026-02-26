use crate::state_machine::stylus::{Context, Event, ID, InputEventHelper, Output};
use crate::state_machine::stylus::{idle::Idle, mmb::MMB, rmb::RMB};
use crate::state_machine::{State, Transition};
use evdev::{EventType, InputEvent, KeyCode};
use log::info;

pub struct LMB;

impl State<Context, Event, Output> for LMB {
    fn enter(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("lmb pressed");
        let ev = self.id().pressed;
        vec![
            InputEvent::new_now(ev.event_type().0, ev.code(), ev.value()),
            InputEvent::new_now(EventType::SYNCHRONIZATION.0, 0, 0),
        ]
    }

    fn exit(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("lmb released");
        let ev = self.id().released;
        vec![
            InputEvent::new_now(ev.event_type().0, ev.code(), ev.value()),
            InputEvent::new_now(EventType::SYNCHRONIZATION.0, 0, 0),
        ]
    }

    fn update(&mut self, ctx: &mut Context, event: Event) -> Transition<Context, Event, Output> {
        if !ctx.pen {
            return Transition::Change(Box::new(Idle), vec![self.id().released]);
        }

        let key = KeyCode::new(event.code());

        match key {
            KeyCode::BTN_STYLUS if ctx.touch => Transition::Change(Box::new(RMB), Vec::new()),
            KeyCode::BTN_STYLUS2 if ctx.touch => Transition::Change(Box::new(MMB), Vec::new()),
            KeyCode::BTN_TOUCH if !ctx.touch => Transition::Change(Box::new(Idle), Vec::new()),
            _ => Transition::Stay(Vec::new()),
        }
    }
}

impl LMB {
    pub fn id(&self) -> ID {
        ID {
            keycode: KeyCode::BTN_TOUCH,
            pressed: InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOUCH.code(), 1),
            released: InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOUCH.code(), 0),
        }
    }
}
