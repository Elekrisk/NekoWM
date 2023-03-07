use std::{ffi::OsString, sync::Arc, os::fd::AsRawFd};

use smithay::{
    desktop::{Space, Window},
    input::{Seat, SeatState},
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, Mode, PostAction},
        wayland_server::Display,
    },
    wayland::{
        compositor::CompositorState,
        data_device::{DataDeviceState, ServerDndGrabHandler, ClientDndGrabHandler},
        output::OutputManagerState,
        shell::{
            kde::decoration::KdeDecorationState,
            xdg::{decoration::XdgDecorationState, XdgShellState},
        },
        shm::{ShmState, ShmHandler},
        socket::ListeningSocketSource, buffer::BufferHandler,
    }, delegate_output, delegate_shm,
};

use crate::{drawing_backend::Backend, CalloopData, window_manager::WindowManager};

pub struct State<B: Backend> {
    pub start_time: std::time::Instant,
    pub socket_name: OsString,

    pub space: Space<Window>,
    pub wm: WindowManager<B>,
    pub loop_signal: LoopSignal,

    pub backend_data: B,

    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub xdg_decoration_state: XdgDecorationState,
    pub kde_decoration_state: KdeDecorationState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,

    pub seat: Seat<Self>,
}

impl<B: Backend> State<B> {
    pub fn new(event_loop: &mut EventLoop<CalloopData<B>>, display: &mut Display<Self>) -> Self {
        let start_time = std::time::Instant::now();

        let dh = display.handle();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&dh);
        let kde_decoration_state = KdeDecorationState::new::<Self>(&dh, smithay::reexports::wayland_protocols_misc::server_decoration::server::org_kde_kwin_server_decoration_manager::Mode::Server);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let mut seat_state = SeatState::new();
        let data_device_state = DataDeviceState::new::<Self>(&dh);

        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, "x11");

        seat.add_keyboard(Default::default(), 200, 200).unwrap();

        seat.add_pointer();

        let mut space = Space::default();

        let socket_name = Self::init_wayland_listener(display, event_loop);

        let loop_signal = event_loop.get_signal();

        let backend_data = B::init(event_loop, display, &mut space);

        std::env::set_var("WAYLAND_DISPLAY", &socket_name);

        Self {
            start_time,
            socket_name,
            space,
            loop_signal,
            backend_data,
            compositor_state,
            xdg_shell_state,
            xdg_decoration_state,
            kde_decoration_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            seat,
        }
    }

    fn init_wayland_listener(
        display: &mut Display<Self>,
        event_loop: &mut EventLoop<CalloopData<B>>,
    ) -> OsString {
        let socket = ListeningSocketSource::new_auto().unwrap();
        let socket_name = socket.socket_name().to_os_string();
        let handle = event_loop.handle();

        handle
            .insert_source(socket, |client_stream, _, state| {
                println!("Client connected!");
                state
                    .display
                    .handle()
                    .insert_client(client_stream, Arc::new(()))
                    .unwrap();
            })
            .unwrap();

        handle
            .insert_source(
                Generic::new(
                    display.backend().poll_fd().as_raw_fd(),
                    Interest::READ,
                    Mode::Level,
                ),
                |_, _, state| {
                    state.display.dispatch_clients(&mut state.state).unwrap();
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }
}

impl<B: Backend> BufferHandler for State<B> {
    fn buffer_destroyed(
        &mut self,
        buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
    }
}

impl<B: Backend> ShmHandler for State<B> {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl<B: Backend> ServerDndGrabHandler for State<B> {}

impl<B: Backend> ClientDndGrabHandler for State<B> {}

delegate_output!(@<B: Backend> State<B>);
delegate_shm!(@<B: Backend> State<B>);
