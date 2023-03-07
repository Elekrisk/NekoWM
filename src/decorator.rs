use smithay::{wayland::shell::{xdg::{decoration::XdgDecorationHandler, ToplevelSurface}, kde::decoration::{KdeDecorationHandler, KdeDecorationState}}, delegate_xdg_decoration, reexports::{wayland_server::{protocol::wl_surface::WlSurface, WEnum}, wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::{Mode as XdgMode}, wayland_protocols_misc::server_decoration::server::org_kde_kwin_server_decoration::{OrgKdeKwinServerDecoration, Mode as KdeMode}}, delegate_kde_decoration};

use crate::{state::State, drawing_backend::Backend};





impl<B: Backend> XdgDecorationHandler for State<B> {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        println!("New XDG decoration");
        toplevel.with_pending_state(|state| {
            // Advertise server side decoration
            state.decoration_mode = Some(XdgMode::ServerSide);
        });
        toplevel.send_configure();
    }

    fn request_mode(
        &mut self,
        toplevel: ToplevelSurface,
        mode: XdgMode,
    ) {
        let s = self.backend_data.size();
        println!("Mode {mode:?} requested");
        toplevel.with_pending_state(|state| {
            state.size = Some(s);
        });
        toplevel.send_configure();
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        println!("Mode unset");
    }
}

delegate_xdg_decoration!(@<B: Backend> State<B>);

impl<B: Backend> KdeDecorationHandler for State<B> {
    fn kde_decoration_state(&self) -> &KdeDecorationState {
        &self.kde_decoration_state
    }

    fn new_decoration(&mut self, _surface: &WlSurface, _decoration: &OrgKdeKwinServerDecoration) {
        println!("Decoration created");
    }

    fn request_mode(
        &mut self,
        _surface: &WlSurface,
        decoration: &OrgKdeKwinServerDecoration,
        mode: WEnum<KdeMode>,
    ) {
        println!("Mode: {mode:?}");
    }

    fn release(&mut self, _decoration: &OrgKdeKwinServerDecoration, _surface: &WlSurface) {
        println!("Decoration released");
    }
}

delegate_kde_decoration!(@<B: Backend> State<B>);
