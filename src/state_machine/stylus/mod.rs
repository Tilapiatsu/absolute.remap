pub mod idle;
pub mod lmb;
pub mod mmb;
pub mod rmb;
use std::time::SystemTime;

use evdev::{AbsoluteAxisCode, InputEvent, KeyCode};
use log::info;

type Output = InputEvent;
type Event = InputEvent;

trait InputEventHelper {
    fn pressed(&self) -> bool;
}

impl InputEventHelper for Event {
    fn pressed(&self) -> bool {
        self.code() != 0
    }
}

trait PressedReleased {
    fn pressed(&self) -> InputEvent;
    fn released(&self) -> InputEvent;
}

pub struct ID {
    keycode: KeyCode,
    pressed: InputEvent,
    released: InputEvent,
}

#[derive(Debug)]
pub struct Context {
    pub pen: bool,
    pub stylus1: bool,
    pub stylus2: bool,
    pub touch: bool,
}

impl Context {
    pub fn new() -> Context {
        Context {
            pen: false,
            stylus1: false,
            stylus2: false,
            touch: false,
        }
    }

    pub fn update_input(&mut self, key: KeyCode, value: i32) {
        let pressed = value != 0;
        // info!("Input : {:?}, {:?}", key, pressed);
        match key {
            KeyCode::BTN_TOOL_PEN => self.pen = pressed, // stylus is in range
            KeyCode::BTN_STYLUS => self.stylus1 = pressed, // stylus button1 is pressed
            KeyCode::BTN_STYLUS2 => self.stylus2 = pressed, // stylus button2 is pressed
            KeyCode::BTN_TOUCH => self.touch = pressed,  // stylus touch the screen
            _ => {}
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}
