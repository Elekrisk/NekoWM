use smithay::{
    desktop::{Space, Window},
    output::Output,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Size}, wayland::shell::xdg::{ToplevelState, ToplevelStateSet, ToplevelSurface}, input::Seat,
};

use crate::{drawing_backend::Backend, state::State};

pub struct WindowManager<B: Backend> {
    output: Output,
    active_layout: Box<dyn Layout<B>>,
}

impl<B: Backend> WindowManager<B> {
    pub fn new(output: Output) -> Self {
        Self {
            output,
            active_layout: Box::new(MonocleLayout {}),
        }
    }

    fn layout(&mut self, space: &Space<Window>) {
        self.active_layout.layout(&self.output, space);
    }

    fn lost_focus(&mut self, state: &Seat<State<B>>, space: &Space<Window>) {
        self.active_layout.lost_focus(state, space);
    }
}

pub trait Layout<B: Backend> {
    fn layout(&mut self, output: &Output, space: &Space<Window>);
    fn lost_focus(&mut self, seat: &Seat<State<B>>, space: &Space<Window>) -> Option<ToplevelSurface>;
}

pub struct MonocleLayout {}

impl<B: Backend> Layout<B> for MonocleLayout {
    fn layout(&mut self, output: &Output, space: &Space<Window>) {
        if space.elements().next().is_none() {
            return;
        }

        let active = space
            .elements()
            .find(|w| {
                w.toplevel()
                    .current_state()
                    .states
                    .contains(xdg_toplevel::State::Activated)
            })
            .cloned();

        let active = if let Some(active) = active {
            active
        } else {
            space.elements().last().unwrap().clone()
        };

        let toplevel = active.toplevel();
        let s = output.current_mode().unwrap().size;
        let size = (s.w.into(), s.h.into()).into();
        toplevel.with_pending_state(|state| state.size = Some(size));
        toplevel.send_configure();
        for element in space
            .elements()
            .filter(|w| **w != active)
            .cloned()
            .collect::<Vec<_>>()
        {
            let toplevel = element.toplevel();
            toplevel.with_pending_state(|state| {
                state.size = Some((0, 0).into());
                state.states = ToplevelStateSet::default();
            });
            space.map_element(element, (-1, -1), false);
        }

        let toplevel = active.toplevel();
        toplevel.with_pending_state(|state| {
            state.size = Some((0, 0).into());
            state.states = ToplevelStateSet::default();
        });
        space.map_element(active, (0, 0), true);
    }

    fn lost_focus(&mut self, seat: &Seat<State<B>>, space: &Space<Window>) -> Option<ToplevelSurface> {
        if space.elements().next().is_none() {
            return None;
        }

        let top = space.elements().last().unwrap().clone();
        top.set_activated(true);
        Some(top.toplevel().clone())
    }
}
