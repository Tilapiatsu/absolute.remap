use std::{cmp::Eq, collections::HashMap};

use crate::state_machine::{StateMachine, stylus};

use anyhow::Result;
use evdev::{
    AbsoluteAxisCode, AttributeSet, Device, EventType, InputEvent, KeyCode, UinputAbsSetup,
    uinput::VirtualDevice,
};
use log::{debug, info, warn};

#[derive(Hash, Eq, PartialEq)]
enum HybridDeviceType {
    Tablet,
    Mouse,
}
struct HybridCollection {
    device_type: HybridDeviceType,
    input_event: InputEvent,
}
struct HybridInputEvents {
    events: Vec<HybridCollection>,
}

impl HybridInputEvents {
    pub fn new() -> Self {
        let events = Vec::new();

        Self { events }
    }

    pub fn push(&mut self, device_type: HybridDeviceType, input_event: InputEvent) {
        &self.events.push(HybridCollection {
            device_type,
            input_event,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

static MOUSE_BTNS: [&str; 3] = ["BTN_LEFT", "BTN_RIGHT", "BTN_MIDDLE"];

static ABS_EVENTS: [&str; 9] = [
    "ABS_TILT_X",
    "ABS_TILT_Y",
    "ABS_X",
    "ABS_Y",
    "ABS_Z",
    "ABS_DISTANCE",
    "ABS_PRESSURE",
    "ABS_WHEEL",
    "ABS_MISC",
];

pub fn remap_evdev(path: &str, debug: &bool, forward: &bool) -> Result<()> {
    let mut device = Device::open(path).unwrap();
    let mut tablet_builder = VirtualDevice::builder()?.name("Vvirtual Cintiq Tablet");
    let mut mouse_builder = VirtualDevice::builder()?.name("Virtual Stylus Mouse");

    println!("Grabbing device: {}", device.name().unwrap_or("unknown"));
    device.grab()?;

    let abs_x = device
        .get_absinfo()
        .unwrap()
        .find(|(axis, _)| *axis == AbsoluteAxisCode::ABS_X)
        .unwrap();

    let abs_y = device
        .get_absinfo()
        .unwrap()
        .find(|(axis, _)| *axis == AbsoluteAxisCode::ABS_Y)
        .unwrap();

    let absolute_axis_x_setup: UinputAbsSetup = UinputAbsSetup::new(abs_x.0, abs_x.1);
    let absolute_axis_y_setup: UinputAbsSetup = UinputAbsSetup::new(abs_y.0, abs_y.1);

    mouse_builder = mouse_builder.with_absolute_axis(&absolute_axis_x_setup)?;
    mouse_builder = mouse_builder.with_absolute_axis(&absolute_axis_y_setup)?;

    if let Some(keys) = device.supported_keys() {
        tablet_builder = tablet_builder.with_keys(keys)?;
    }

    let mut keys: AttributeSet<KeyCode> = AttributeSet::new();
    for code in KeyCode::KEY_RESERVED.code()..KeyCode::BTN_TRIGGER_HAPPY40.code() {
        let key = KeyCode::new(code);
        let name = format!("{key:?}");
        if MOUSE_BTNS.contains(&&*name) {
            keys.insert(key);
            if *debug {
                debug!("Register {:?}", name);
            }
        }
    }

    mouse_builder = mouse_builder.with_keys(&keys)?;

    #[warn(clippy::collapsible_if)]
    if *forward {
        if let Some(abs_axes) = device.supported_absolute_axes() {
            for axis in abs_axes {
                let axe = format!("{axis:?}");
                if !ABS_EVENTS.contains(&&*axe) {
                    warn!("Skipping {:?}", axe);
                    continue;
                }
                let absinfo = device
                    .get_absinfo()?
                    .find(|(absolut_axis_code, _)| *absolut_axis_code == axis)
                    .map(|(_, abs_info)| abs_info)
                    .unwrap();
                info!("{:?} = {:?}", axis, absinfo);
                let absolute_axis_setup: UinputAbsSetup = UinputAbsSetup::new(axis, absinfo);
                tablet_builder = tablet_builder.with_absolute_axis(&absolute_axis_setup)?;
            }
        }

        if let props = device.properties() {
            tablet_builder = tablet_builder.with_properties(props)?;
        }

        if let Some(misc) = device.misc_properties() {
            info!("{:?}", misc);
            tablet_builder = tablet_builder.with_msc(misc)?;
        }
    }

    let mut virtual_tablet = tablet_builder.build()?;
    // let mut virtual_mouse = mouse_builder.build()?;

    let mut context = stylus::Context::new();
    let mut stylus_sm = StateMachine::new(Box::new(stylus::idle::Idle));

    info!("Virtual device created.");

    loop {
        let mut m_batch: Vec<InputEvent> = Vec::new();

        let mut t_batch: Vec<InputEvent> = Vec::new();
        let mut o_batch: Vec<InputEvent> = Vec::new();

        for ev in device.fetch_events()? {
            match ev.event_type() {
                EventType::KEY => {
                    let key = KeyCode::new(ev.code());

                    context.update_input(key, ev.value());

                    let outputs = stylus_sm.handle_event(&mut context, ev);

                    if !outputs.is_empty() {
                        virtual_tablet.emit(&outputs)?;
                        // if *debug {
                        //     for o in &outputs {
                        //         debug!("{:?}", *o);
                        //     }
                        // }
                        for o in outputs {
                            m_batch.push(o);
                        }
                        //     for o in &outputs {
                        //         batch.push(HybridDeviceType::Mouse, *o);
                        //     }
                    }

                    continue;
                }
                // EventType::ABSOLUTE => {
                //     let code = ev.code();
                //     if code == AbsoluteAxisCode::ABS_X.0 {
                //         context.update_pos(AbsoluteAxisCode::ABS_X, ev.value());
                //     } else if code == AbsoluteAxisCode::ABS_Y.0 {
                //         context.update_pos(AbsoluteAxisCode::ABS_Y, ev.value());
                //     }
                //     if code == AbsoluteAxisCode::ABS_X.0 || code == AbsoluteAxisCode::ABS_Y.0 {
                //         let event = InputEvent::new_now(ev.event_type().0, ev.code(), ev.value());
                //         m_batch.push(event);
                //     }
                //     t_batch.push(ev);
                // }
                // -----------------------------
                // Forward everything else
                // -----------------------------
                _ => {
                    if !*forward {
                        continue;
                    }
                    let forwarded = InputEvent::new_now(ev.event_type().0, ev.code(), ev.value());

                    t_batch.push(forwarded);
                }
            }
            if ev.event_type() == EventType::SYNCHRONIZATION {
                // batch.push(InputEvent::new(ev.event_type().0, ev.code(), ev.value()));
                if !t_batch.is_empty() {
                    virtual_tablet.emit(&t_batch)?;
                    if *debug {
                        for b in t_batch.iter() {
                            debug!("Emitting Event : {:?}", b);
                        }
                        debug!("-------------- SYN_REPORT ------------");
                    }
                    t_batch.clear();
                }
                //     if !m_batch.is_empty() {
                //         virtual_mouse.emit(&m_batch)?;
                //
                //         if *debug {
                //             for b in m_batch.iter() {
                //                 debug!("Emitting Event : {:?}", b);
                //             }
                //             debug!("-------------- SYN_REPORT ------------");
                //         }
                //         m_batch.clear();
                //     }
            }
        }
    }
}
