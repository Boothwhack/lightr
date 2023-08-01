# Lightr
Tiny utility to control backlight brightness on Linux. Lightr utilizes sysfs to query backlight properties and modifies
them using logind's D-Bus api. Brightness values are produced on an exponential curve to provide more granular control 
in the lower end of the brightness range.

This is still a work in progress, and as such most values and paths are hardcoded.

## Usage
```
lightr [up|down]
```
It's that simple.

## Building
Lightr is written in Rust, and so uses Cargo as its build system. To build a release build, simply run:
```
cargo build --release
```
The binary will be located at `target/release/lightr`.
