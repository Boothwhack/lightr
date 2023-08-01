use std::error::Error;
use std::ops::RangeInclusive;
use std::path::Path;
use std::process;
use std::str::FromStr;
use async_std::fs::File;
use async_std::prelude::*;

enum Function {
    Exponential(f32),
}

impl Function {
    fn next(&self, current: u32, step: f32, max: u32, direction: Direction) -> u32 {
        match self {
            Function::Exponential(power) => {
                let current = (current as f32) / (max as f32);
                let current = current.powf(1.0 / power);
                let current = current + step * direction.sign() as f32;
                if current < 0.0 {
                    return 0;
                }
                let current = current.powf(*power);
                (current * max as f32) as u32
            }
        }
    }
}

enum Direction {
    Up,
    Down,
}

impl Direction {
    fn sign(&self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
        }
    }
}

struct Configuration {
    brightness_rng: RangeInclusive<u32>,
    current: u32,
    step: f32,
    function: Function,
}

async fn read_int_from_file(path: impl AsRef<Path>) -> Result<u32, Box<dyn Error>> {
    let mut file = File::open(path.as_ref()).await?;
    let mut value = String::new();
    file.read_to_string(&mut value).await?;
    Ok(u32::from_str(value.trim())?)
}

impl Configuration {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let brightness = read_int_from_file("/sys/class/backlight/intel_backlight/brightness").await?;
        let max_brightness = read_int_from_file("/sys/class/backlight/intel_backlight/max_brightness").await?;
        let min_brightness = (0.003 * max_brightness as f32) as u32;

        Ok(Self {
            brightness_rng: min_brightness..=max_brightness,
            current: brightness,
            step: 0.05,
            function: Function::Exponential(3.0),
        })
    }

    fn min(&self) -> u32 {
        *self.brightness_rng.start()
    }

    fn max(&self) -> u32 {
        *self.brightness_rng.end()
    }

    fn step(&mut self, direction: Direction) {
        let next = self.function.next(self.current - self.min(), self.step, self.max() - self.min(), direction);
        let next = next + self.min();
        self.current = next.clamp(self.min(), self.max());
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let direction = match std::env::args().skip(1).next().as_ref().map(String::as_str) {
        Some("up") => Direction::Up,
        Some("down") => Direction::Down,
        _ => {
            eprintln!("Usage: {} [up|down]", std::env::args().next().unwrap());
            process::exit(1);
        }
    };

    let mut config = Configuration::new().await?;
    config.step(direction);

    let connection = zbus::Connection::system().await?;
    let session = logind_zbus::session::SessionProxy::builder(&connection).path("/org/freedesktop/login1/session/auto")?.build().await?;
    session.set_brightness("backlight", "intel_backlight", config.current).await?;

    Ok(())
}
