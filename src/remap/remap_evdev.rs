use crate::state_machine::{StateMachine, stylus};

use anyhow::Result;
use evdev::{Device, EventType, InputEvent, KeyCode, UinputAbsSetup, uinput::VirtualDevice};
use log::{debug, info, warn};

pub fn remap_evdev(path: &str, debug: &bool, forward: &bool) -> Result<()> {
    let mut device = Device::open(path).unwrap();
    let mut tablet_builder = VirtualDevice::builder()?.name("Virtual Cintiq Tablet");

    println!("Grabbing device: {}", device.name().unwrap_or("unknown"));
    device.grab()?;

    let input_id = device.input_id();
    tablet_builder = tablet_builder.input_id(input_id);

    if let props = device.properties() {
        tablet_builder = tablet_builder.with_properties(props)?;
    }

    if let Some(keys) = device.supported_keys() {
        tablet_builder = tablet_builder.with_keys(keys)?;
    }

    if let Some(abs_axes) = device.supported_absolute_axes() {
        for axis in abs_axes {
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

    if let Some(misc) = device.misc_properties() {
        info!("{:?}", misc);
        tablet_builder = tablet_builder.with_msc(misc)?;
    }

    if let Some(tools) = device.supported_switches() {
        info!("{:?}", tools);
        tablet_builder = tablet_builder.with_switches(tools)?;
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
                    if [
                        KeyCode::BTN_TOOL_PEN,
                        KeyCode::BTN_TOUCH,
                        KeyCode::BTN_STYLUS,
                        KeyCode::BTN_STYLUS2,
                    ]
                    .contains(&key)
                    {
                        let outputs = stylus_sm.handle_event(&mut context, ev);

                        if !outputs.is_empty() {
                            //NOTE:  Krita needs keys input events to be emitted right away :
                            // virtual_tablet.emit(&outputs)?;
                            for o in outputs {
                                t_batch.push(o);
                            }
                        }
                    } else {
                        t_batch.push(ev);
                        // virtual_tablet.emit(&[ev])?;
                    }
                }
                EventType::ABSOLUTE => {
                    t_batch.push(ev);
                }
                // -----------------------------
                // Forward everything else
                // -----------------------------
                _ => {
                    if !*forward {
                        continue;
                    }

                    t_batch.push(ev);
                }
            }

            if ev.event_type() == EventType::SYNCHRONIZATION {
                // batch.push(InputEvent::new(ev.event_type().0, ev.code(), ev.value()));
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
