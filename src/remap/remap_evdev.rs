use crate::state_machine::{StateMachine, stylus};

use anyhow::Result;
use evdev::{
    AttributeSet, Device, EventType, InputEvent, KeyCode, UinputAbsSetup, uinput::VirtualDevice,
};
use log::{debug, info, warn};

static MOUSE_BTNS: [&str; 20] = [
    "BTN_MISC",
    "BTN_0",
    "BTN_1",
    "BTN_2",
    "BTN_3",
    "BTN_4",
    "BTN_5",
    "BTN_6",
    "BTN_7",
    "BTN_8",
    "BTN_9",
    "BTN_MOUSE",
    "BTN_LEFT",
    "BTN_RIGHT",
    "BTN_MIDDLE",
    "BTN_SIDE",
    "BTN_EXTRA",
    "BTN_FORWARD",
    "BTN_BACK",
    "BTN_TASK",
];

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
    let mut builder = VirtualDevice::builder()?.name("Cintiq Stylus Proxy");

    println!("Grabbing device: {}", device.name().unwrap_or("unknown"));
    device.grab()?;

    if let Some(keys) = device.supported_keys() {
        builder = builder.with_keys(keys)?;
    }
    let mut keys: AttributeSet<KeyCode> = AttributeSet::new();
    for code in KeyCode::KEY_RESERVED.code()..KeyCode::BTN_TRIGGER_HAPPY40.code() {
        let key = KeyCode::new(code);
        let name = format!("{key:?}");
        if name.starts_with("KEY_") || MOUSE_BTNS.contains(&&*name) {
            keys.insert(key);
            if *debug {
                debug!("Register {:?}", name);
            }
        }
    }

    builder = builder.with_keys(&keys)?;

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
                builder = builder.with_absolute_axis(&absolute_axis_setup)?;
            }
        }

        if let Some(misc) = device.misc_properties() {
            info!("{:?}", misc);
            builder = builder.with_msc(misc)?;
        }
    }

    let mut virtual_device = builder.build()?;

    let mut context = stylus::Context::new();
    let mut stylus_sm = StateMachine::new(Box::new(stylus::idle::Idle));

    info!("Virtual device created.");

    loop {
        let mut batch: Vec<InputEvent> = Vec::new();
        for ev in device.fetch_events()? {
            match ev.event_type() {
                EventType::KEY => {
                    let key = KeyCode::new(ev.code());

                    context.update(key, ev.value());
                    // debug!("{:?}", ev);

                    let outputs = stylus_sm.handle_event(&mut context, ev);

                    if !outputs.is_empty() {
                        for o in &outputs {
                            debug!("{:?}", o);
                        }

                        virtual_device.emit(&outputs)?;
                    }
                }
                // -----------------------------
                // Forward everything else
                // -----------------------------
                _ => {
                    if !*forward {
                        continue;
                    }
                    // Rebuild clean event (no timestamp)
                    let forwarded = InputEvent::new(ev.event_type().0, ev.code(), ev.value());
                    if *debug {
                        debug!("Forwarding {:?}", forwarded);
                    }
                    batch.push(forwarded);
                    // virtual_dev.emit(&[forwarded]).unwrap();
                }
            }
            if ev.event_type() == EventType::SYNCHRONIZATION {
                virtual_device.emit(&batch)?;
                batch.clear();
            }
        }
    }
}
