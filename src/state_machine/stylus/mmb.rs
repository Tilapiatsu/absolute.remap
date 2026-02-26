use crate::state_machine::stylus::{Context, Event, ID, InputEventHelper, Output};
use crate::state_machine::stylus::{idle::Idle, lmb::LMB};
use crate::state_machine::{State, Transition};
use evdev::{EventType, InputEvent, KeyCode};
use log::info;

pub struct MMB;

impl State<Context, Event, Output> for MMB {
    fn enter(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("mmb pressed");
        let ev = self.id().pressed;
        vec![
            // InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOUCH.code(), 1),
            InputEvent::new_now(ev.event_type().0, ev.code(), ev.value()),
            // InputEvent::new_now(EventType::SYNCHRONIZATION.0, 0, 0),
        ]
    }

    fn exit(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("mmb released");
        let ev = self.id().released;
        vec![
            // InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOUCH.code(), 0),
            InputEvent::new_now(ev.event_type().0, ev.code(), ev.value()),
            // InputEvent::new_now(EventType::SYNCHRONIZATION.0, 0, 0),
        ]
    }

    fn update(&mut self, ctx: &mut Context, event: Event) -> Transition<Context, Event, Output> {
        if !ctx.pen {
            return Transition::Change(Box::new(Idle), vec![self.id().released]);
        }

        let key = KeyCode::new(event.code());

        match key {
            KeyCode::BTN_STYLUS2 if ctx.touch => Transition::Change(Box::new(LMB), Vec::new()),
            KeyCode::BTN_TOUCH if !ctx.touch => Transition::Change(Box::new(Idle), Vec::new()),
            _ => Transition::Stay(Vec::new()),
        }
    }
}

impl MMB {
    pub fn id(&self) -> ID {
        ID {
            keycode: KeyCode::BTN_STYLUS,
            pressed: InputEvent::new(EventType::KEY.0, KeyCode::BTN_STYLUS.code(), 1),
            released: InputEvent::new(EventType::KEY.0, KeyCode::BTN_STYLUS.code(), 0),
        }
    }
}
