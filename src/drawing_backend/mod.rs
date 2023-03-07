use smithay::{
    desktop::{Space, Window},
    reexports::{calloop::EventLoop, wayland_server::Display}, utils::{Size, Physical, Logical},
};

use crate::{CalloopData, state::State};

pub mod x11;

pub trait Backend: Sized + 'static {
    fn init(
        event_loop: &mut EventLoop<CalloopData<Self>>,
        display: &mut Display<State<Self>>,
        space: &mut Space<Window>,
    ) -> Self;

    fn draw(data: &mut CalloopData<Self>);

    fn size(&self) -> Size<i32, Logical>;
}
