use smithay::{wayland::shell::xdg::{XdgShellHandler, XdgShellState, ToplevelSurface, PopupSurface, PositionerState, ShellClient, Configure}, utils::{SERIAL_COUNTER, Serial, Point, Logical}, desktop::Window, delegate_xdg_shell, reexports::{wayland_server::protocol::{wl_seat::WlSeat, wl_output::WlOutput, wl_surface::WlSurface}, wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge}};

use crate::{state::State, drawing_backend::Backend};

impl<B: Backend> XdgShellHandler for State<B> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let serial = SERIAL_COUNTER.next_serial();
        self.seat.get_keyboard().unwrap().set_focus(
            self,
            Some(surface.wl_surface().clone()),
            serial,
        );
        let window = Window::new(surface);
        self.space.map_element(window, (0, 0), true);
        for elem in self.space.elements() {
            let toplevel = elem.toplevel();
            let current_state = toplevel.current_state();

            if elem.toplevel().with_pending_state(|state| *state != current_state) {
                elem.toplevel().send_configure();
            }
        }
    }

    fn new_popup(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
    ) {
        todo!()
    }

    fn grab(
        &mut self,
        surface: PopupSurface,
        seat: WlSeat,
        serial: Serial,
    ) {
        todo!()
    }

    fn new_client(&mut self, client: ShellClient) {}

    fn client_pong(&mut self, client: ShellClient) {}

    fn move_request(&mut self, surface: ToplevelSurface, seat: WlSeat, serial: Serial) {}

    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        seat: WlSeat,
        serial: Serial,
        edges: ResizeEdge,
    ) {
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {}

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {}

    fn fullscreen_request(&mut self, surface: ToplevelSurface, output: Option<WlOutput>) {}

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {}

    fn minimize_request(&mut self, surface: ToplevelSurface) {}

    fn show_window_menu(
        &mut self,
        surface: ToplevelSurface,
        seat: WlSeat,
        serial: Serial,
        location: Point<i32, Logical>,
    ) {
    }

    fn ack_configure(&mut self, surface: WlSurface, configure: Configure) {}

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {}

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let window = self.space.elements().find(|w| *w.toplevel() == surface).cloned();
        if let Some(window) = window {
            self.space.unmap_elem(&window);
        }
        println!("Space contains {} windows", self.space.elements().count());
        if let Some(kb) = self.seat.get_keyboard() && kb.current_focus() == Some(surface.wl_surface().clone()) {
            self;
        }
    }

    fn popup_destroyed(&mut self, surface: PopupSurface) {}


}

delegate_xdg_shell!(@<B: Backend> State<B>);
