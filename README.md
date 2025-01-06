# Trackball Scroll Emulation

A small Rust program that repurposes a trackball’s back button (BTN_SIDE) into a scroll toggle. When you press and hold the back button, moving the trackball will produce scroll events instead of pointer movements. Releasing the back button returns it to normal movement mode.

## Motivation

Many trackball devices lack a dedicated scroll wheel, forcing you to use a separate method (like a scroll ring or on-screen scrollbar). By intercepting the physical back button and using it to toggle “scroll mode,” this program makes your trackball more versatile and efficient:

- **Convenience**: No need for separate scroll rings, buttons, or keyboard shortcuts for scrolling.  
- **Customization**: Easilyt adjust scrolling speed and pointer movement rate to your preference.

## How It Works

1. **Grab the physical device** via [evdev] so the back button doesn’t trigger a browser or window manager “Back” action.  
2. **Create a virtual device** via [uinput], which the OS sees as a legitimate mouse with movement, scroll, and click capabilities.  
3. **Intercept input events** from the real trackball:
   - **Back Button Press**: Toggles *scroll mode* on or off.  
   - **Pointer Movement**: 
     - If *scroll mode* is **off**, forward movement events (scaled by a `MOVE_RATE`) to the virtual device as normal pointer movement.  
     - If *scroll mode* is **on**, transform the movement events into vertical and horizontal scroll events (with optional fractional accumulation for smoother scrolling).  
   - **Click Events**: Forward BTN_LEFT and BTN_RIGHT events to the virtual device so that clicking still works as expected.

## Requirements

- **Linux** (with `/dev/input` and `/dev/uinput`)  
- **Rust** (1.60+ recommended)  
- Permissions to read from `/dev/input/event*` and write to `/dev/uinput` (commonly root or special udev rules)  

## Usage

1. **Build**  
   ```bash
   cargo build --release
   ```
2. **Run (as root)**  
   ```bash
   sudo ./target/release/trackball-scroll
   ```
3. **Move** the trackball normally.  
4. **Press** the back button to toggle scroll mode; move the trackball to scroll.  
5. **Release** the back button to revert to normal pointer movement.

## Systemd Service Setup

If you want this to run automatically at boot:

1. Copy the compiled binary to a system location, e.g.:
   ```bash
   sudo cp target/release/trackball-scroll /usr/local/bin/trackball-scroll
   ```
2. Create a systemd unit file `/etc/systemd/system/trackball-scroller.service`:
   ```ini
   [Unit]
   Description=Trackball Scroll Emulation
   After=multi-user.target

   [Service]
   Type=simple
   ExecStart=/usr/local/bin/trackball-scroll
   Restart=always
   User=root
   Group=root

   [Install]
   WantedBy=multi-user.target
   ```
3. Enable and start the service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable trackball-scroller.service
   sudo systemctl start trackball-scroller.service
   ```
4. Check status:
   ```bash
   systemctl status trackball-scroller.service
   ```
   
## Configuration

- **MOVE_RATE**: Adjust the pointer speed for normal movement.  
- **SCROLL_FACTOR**: Adjust how quickly scrolling accumulates ticks. Decrease for smoother/slower scrolling, increase for faster/more immediate scrolling.  
- **Additional Buttons**: Forward other buttons or shortcuts by matching the appropriate `Key::BTN_*` events.

**Happy Scrolling!**
