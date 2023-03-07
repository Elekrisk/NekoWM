use smithay::{wayland::{compositor::{CompositorHandler, CompositorState, is_sync_subsurface, get_parent, with_states}, shell::xdg::XdgToplevelSurfaceData}, reexports::wayland_server::protocol::wl_surface::WlSurface, backend::renderer::utils::on_commit_buffer_handler, delegate_compositor};

use crate::{state::State, drawing_backend::Backend};



impl<B: Backend> CompositorHandler for State<B> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self
                .space
                .elements()
                .find(|w| w.toplevel().wl_surface() == &root)
            {
                window.on_commit();
            }
        }

        if let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()
        {
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            });

            if !initial_configure_sent {
                println!("Sending initial configure");
                let s = self.backend_data.size();
                let toplevel = window.toplevel();
                toplevel.with_pending_state(|state| {
                    state.size = Some(s);
                    state.decoration_mode = Some(smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide);
                });
                toplevel.send_configure();
            }
        }
    }
}

delegate_compositor!(@<B: Backend> State<B>);
