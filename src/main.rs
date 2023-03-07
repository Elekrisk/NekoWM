#![feature(let_chains)]

pub mod compositor;
pub mod data_device;
pub mod decorator;
pub mod drawing_backend;
mod input;
pub mod shell;
pub mod state;
pub mod window_manager;

use drawing_backend::x11::X11BackendData;
use drawing_backend::Backend;
use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
use state::State;
use std::time::Duration;
use tracing::metadata::LevelFilter;

pub struct CalloopData<B: Backend> {
    state: State<B>,
    display: Display<State<B>>,
}

fn main() {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(LevelFilter::INFO)
        .init();

    let mut event_loop: EventLoop<CalloopData<X11BackendData>> = EventLoop::try_new().unwrap();
    let mut display: Display<State<X11BackendData>> = Display::new().unwrap();
    let state = State::new(&mut event_loop, &mut display);
    let mut data = CalloopData { state, display };

    if let Some(command) = std::env::args().skip(1).next() {
        std::process::Command::new(&command).spawn().unwrap();
    }

    event_loop
        .run(
            Duration::from_secs_f32(1.0 / 60.0),
            &mut data,
            move |data| {
                X11BackendData::draw(data);
            },
        )
        .unwrap();
}
