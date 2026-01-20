use evdev::{
    uinput, AttributeSet, Device, EventType, InputEvent, InputEventKind, Key, RelativeAxisType,
};
use std::io::{Error, ErrorKind, Result};

/// Pointer speed multiplier for normal movement mode.
const MOVE_RATE: f32 = 1.5;

/// Scroll sensitivity factor. Lower values produce smoother/slower scrolling.
const SCROLL_FACTOR: f32 = 0.2;

/// Path to the trackball input device.
const DEVICE_PATH: &str = "/dev/input/by-id/usb-Logitech_USB_Trackball-event-mouse";

fn main() -> Result<()> {
    let mut trackball = Device::open(DEVICE_PATH).map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "Failed to open trackball device at '{}': {}. \
                Ensure the device is connected and you have permission to access it (try running as root).",
                DEVICE_PATH, e
            ),
        )
    })?;

    trackball.grab().map_err(|e| {
        Error::new(
            ErrorKind::PermissionDenied,
            format!(
                "Failed to grab exclusive access to trackball: {}. \
                Another program may be using the device, or you may need root permissions.",
                e
            ),
        )
    })?;

    let mut virtual_device = uinput::VirtualDeviceBuilder::new()
        .map_err(|e| {
            Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "Failed to create virtual device builder: {}. \
                    Ensure /dev/uinput is accessible (try running as root).",
                    e
                ),
            )
        })?
        .name("Virtual Scroll Device")
        .with_relative_axes(&AttributeSet::from_iter([
            RelativeAxisType::REL_X,
            RelativeAxisType::REL_Y,
            RelativeAxisType::REL_WHEEL,
            RelativeAxisType::REL_HWHEEL,
        ]))
        .map_err(|e| Error::other(format!("Failed to configure relative axes: {}", e)))?
        .with_keys(&AttributeSet::from_iter([
            Key::BTN_LEFT,
            Key::BTN_RIGHT,
            Key::BTN_MIDDLE,
        ]))
        .map_err(|e| Error::other(format!("Failed to configure key events: {}", e)))?
        .build()
        .map_err(|e| Error::other(format!("Failed to build virtual device: {}", e)))?;

    eprintln!("Trackball scroll emulation started. Press the back button to toggle scroll mode.");

    let mut scroll_mode = false;

    // Accumulators for partial scrolling
    let mut accum_x: f32 = 0.0;
    let mut accum_y: f32 = 0.0;

    loop {
        for ev in trackball.fetch_events()? {
            match ev.kind() {
                InputEventKind::Key(Key::BTN_SIDE) => {
                    scroll_mode = ev.value() == 1;
                }
                InputEventKind::Key(Key::BTN_LEFT) | InputEventKind::Key(Key::BTN_RIGHT) => {
                    virtual_device.emit(&[InputEvent::new_now(
                        EventType::KEY,
                        ev.code(),
                        ev.value(),
                    )])?;
                }
                InputEventKind::Key(Key::BTN_EXTRA) => {
                    virtual_device.emit(&[InputEvent::new_now(
                        EventType::KEY,
                        Key::code(Key::BTN_MIDDLE),
                        ev.value(),
                    )])?;
                }
                InputEventKind::RelAxis(axis) => {
                    match axis {
                        _ if scroll_mode
                            && (axis == RelativeAxisType::REL_X
                                || axis == RelativeAxisType::REL_Y) =>
                        {
                            let delta = ev.value();

                            if axis == RelativeAxisType::REL_X {
                                // Accumulate horizontal movement
                                accum_x += (delta as f32) * SCROLL_FACTOR;
                                let ticks_x = accum_x as i32;
                                if ticks_x != 0 {
                                    virtual_device.emit(&[InputEvent::new_now(
                                        EventType::RELATIVE,
                                        RelativeAxisType::REL_HWHEEL.0,
                                        ticks_x,
                                    )])?;
                                    accum_x -= ticks_x as f32; // keep the remainder
                                }
                            } else {
                                // Accumulate vertical movement
                                accum_y += (delta as f32) * SCROLL_FACTOR;
                                let ticks_y = accum_y as i32;
                                if ticks_y != 0 {
                                    virtual_device.emit(&[InputEvent::new_now(
                                        EventType::RELATIVE,
                                        RelativeAxisType::REL_WHEEL.0,
                                        ticks_y,
                                    )])?;
                                    accum_y -= ticks_y as f32;
                                }
                            }
                        }

                        RelativeAxisType::REL_X => {
                            let dx = (ev.value() as f32 * MOVE_RATE) as i32;
                            virtual_device.emit(&[InputEvent::new_now(
                                EventType::RELATIVE,
                                RelativeAxisType::REL_X.0,
                                dx,
                            )])?;
                        }
                        RelativeAxisType::REL_Y => {
                            let dy = (ev.value() as f32 * MOVE_RATE) as i32;
                            virtual_device.emit(&[InputEvent::new_now(
                                EventType::RELATIVE,
                                RelativeAxisType::REL_Y.0,
                                dy,
                            )])?;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
