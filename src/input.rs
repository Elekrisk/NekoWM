use smithay::{
    backend::input::{Event, InputBackend, InputEvent, KeyboardKeyEvent},
    delegate_seat,
    input::{
        keyboard::{FilterResult, KeyboardTarget},
        pointer::CursorImageStatus,
        Seat, SeatHandler, SeatState,
    },
    reexports::{wayland_server::protocol::wl_surface::WlSurface, wayland_protocols::xdg::shell::server::xdg_toplevel},
    utils::SERIAL_COUNTER,
    wayland::shell::xdg::ToplevelSurface,
};

use crate::{drawing_backend::Backend, state::State, CalloopData};

impl<B: Backend> State<B> {
    pub fn event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::Keyboard { event } => {
                let serial = smithay::utils::SERIAL_COUNTER.next_serial();
                let time = Event::time_msec(&event);

                for (i, elem) in self.space.elements().enumerate() {
                    if elem.toplevel().current_state().states.contains(xdg_toplevel::State::Activated) {
                        print!("{i}.");
                    } else {
                        print!("{i}!");
                    }
                }
                println!();

                self.seat.get_keyboard().unwrap().input::<(), _>(
                    self,
                    event.key_code(),
                    event.state(),
                    serial,
                    time,
                    |a, b, c| FilterResult::Forward,
                );
            }
            _ => {}
        }
    }
}

impl<B: Backend> SeatHandler for State<B> {
    type KeyboardFocus = WlSurface;

    type PointerFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>) {
        println!("Focus changed to {focused:?}");
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

delegate_seat!(@<B: Backend> State<B>);
