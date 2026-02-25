use crate::state_machine::stylus::{Context, Event, InputEventHelper, Output};
use crate::state_machine::stylus::{lmb::LMB, mmb::MMB, rmb::RMB};
use crate::state_machine::{State, Transition};
use evdev::{InputEvent, KeyCode};
use log::info;

pub struct Idle;

impl State<Context, Event, Output> for Idle {
    fn enter(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("enter idle");
        Vec::new()
    }

    fn exit(&mut self, _ctx: &mut Context) -> Vec<Output> {
        info!("exit idle");
        Vec::new()
    }

    fn update(&mut self, ctx: &mut Context, event: Event) -> Transition<Context, Event, Output> {
        if !ctx.pen {
            return Transition::Stay(Vec::new());
        }

        let key = KeyCode::new(event.code());

        match key {
            KeyCode::BTN_STYLUS if !ctx.touch => {
                ctx.stylus1 = event.pressed();
                Transition::Stay(Vec::new())
            }
            KeyCode::BTN_STYLUS2 if !ctx.touch => {
                ctx.stylus2 = event.pressed();
                Transition::Stay(Vec::new())
            }
            KeyCode::BTN_TOUCH if ctx.stylus1 => {
                ctx.touch = event.pressed();

                Transition::Change(
                    Box::new(RMB),
                    vec![
                        InputEvent::new(1, KeyCode::BTN_TOUCH.code(), 1),
                        InputEvent::new(1, KeyCode::BTN_TOOL_PEN.code(), 1),
                    ],
                )
            }
            KeyCode::BTN_TOUCH if ctx.stylus2 => {
                ctx.touch = event.pressed();
                Transition::Change(
                    Box::new(MMB),
                    vec![
                        InputEvent::new(1, KeyCode::BTN_TOUCH.code(), 1),
                        InputEvent::new(1, KeyCode::BTN_TOOL_PEN.code(), 1),
                    ],
                )
            }
            KeyCode::BTN_TOUCH if !ctx.stylus1 && !ctx.stylus2 => {
                ctx.touch = true;
                Transition::Change(Box::new(LMB), Vec::new())
            }
            _ => Transition::Stay(Vec::new()),
        }
    }
}
