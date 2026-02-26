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

    println!("Grabbing device: {}", device.name().unwrap_or("unknown"));
    device.grab()?;

    if let Some(keys) = device.supported_keys() {
        tablet_builder = tablet_builder.with_keys(keys)?;
    }

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

    let mut context = stylus::Context::new();
    let mut stylus_sm = StateMachine::new(Box::new(stylus::idle::Idle));

    info!("Virtual device created.");

    loop {
        let mut t_batch: Vec<InputEvent> = Vec::new();

        for ev in device.fetch_events()? {
            match ev.event_type() {
                EventType::KEY => {
                    let key = KeyCode::new(ev.code());

                    context.update_input(key, ev.value());

                    let outputs = stylus_sm.handle_event(&mut context, ev);

                    if !outputs.is_empty() {
                        virtual_tablet.emit(&outputs)?;
                        for o in outputs {
                            t_batch.push(o);
                        }
                    }

                    continue;
                }
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
            }
        }
    }
}
