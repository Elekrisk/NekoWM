use std::{collections::HashSet, time::Duration};

use smithay::{
    backend::{
        allocator::{
            dmabuf::DmabufAllocator,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
        },
        egl::{EGLContext, EGLDisplay},
        renderer::{damage::DamageTrackedRenderer, gles2::Gles2Renderer, element::surface::WaylandSurfaceRenderElement, Bind},
        x11::{Window as X11Window, WindowBuilder, X11Backend, X11Event, X11Surface},
    },
    desktop::{Space, Window},
    output::{Output, PhysicalProperties, Subpixel},
    reexports::{calloop::EventLoop, wayland_server::Display},
    utils::{DeviceFd, Size, Physical, Logical},
};

use crate::{CalloopData, state::State};

use super::Backend;

pub struct X11BackendData {
    window: X11Window,
    surface: X11Surface,
    renderer: Gles2Renderer,
    damage_tracked_renderer: DamageTrackedRenderer,
    output: Output,
}

fn x11_draw(data: &mut CalloopData<X11BackendData>) {
    let backend_data = &mut data.state.backend_data;
    let surface = &mut backend_data.surface;
    let renderer = &mut backend_data.renderer;
    let damage_tracked_renderer = &mut backend_data.damage_tracked_renderer;
    let output = &backend_data.output;

    let (buffer, age) = surface.buffer().unwrap();
    renderer.bind(buffer).unwrap();
    smithay::desktop::space::render_output::<_, WaylandSurfaceRenderElement<Gles2Renderer>, _, _>(
        output,
        renderer,
        0,
        [&data.state.space],
        &[],
        damage_tracked_renderer,
        [0.5, 0.5, 0.5, 1.0],
    )
    .unwrap();
    surface.submit().unwrap();

    data.state.space.elements().for_each(|window| {
        window.send_frame(
            &output,
            data.state.start_time.elapsed(),
            Some(Duration::ZERO),
            |_, _| Some(output.clone()),
        )
    });

    data.state.space.refresh();
    data.display.flush_clients().unwrap();
}

pub fn init_x11(
    event_loop: &mut EventLoop<CalloopData<X11BackendData>>,
    display: &mut Display<State<X11BackendData>>,
    space: &mut Space<Window>,
) -> X11BackendData {
    let backend = X11Backend::new().unwrap();
    let x_handle = backend.handle();
    let window = WindowBuilder::new()
        .title("NekoWM")
        .build(&x_handle)
        .unwrap();
    let (_drm_node, fd) = x_handle.drm_node().unwrap();
    let device = GbmDevice::new(DeviceFd::from(fd)).unwrap();
    let egl = EGLDisplay::new(device.clone()).unwrap();
    let context = EGLContext::new(&egl).unwrap();
    let modifiers = context
        .dmabuf_render_formats()
        .iter()
        .map(|format| format.modifier)
        .collect::<HashSet<_>>();
    let surface = x_handle
        .create_surface(
            &window,
            DmabufAllocator(GbmAllocator::new(device, GbmBufferFlags::RENDERING)),
            modifiers.into_iter(),
        )
        .unwrap();

    let mut renderer = unsafe { Gles2Renderer::new(context) }.unwrap();

    let size = {
        let s = window.size().to_physical(1);
        (s.w.into(), s.h.into()).into()
    };

    let mode = smithay::output::Mode {
        size,
        refresh: 60_000,
    };

    let output = Output::new(
        "weeeeee".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "NekoWM".into(),
            model: "x11".into(),
        },
    );

    output.create_global::<State<X11BackendData>>(&display.handle());
    output.change_current_state(Some(mode), None, None, Some((0, 0).into()));
    output.set_preferred(mode);

    space.map_output(&output, (0, 0));

    let mut damage_tracked_renderer = DamageTrackedRenderer::from_output(&output);

    let mut full_redraw = 0u8;

    let signal = event_loop.get_signal();

    event_loop
        .handle()
        .insert_source(backend, move |event, _, data| match event {
            X11Event::Refresh { window_id } => {}
            X11Event::Input(event) => data.state.event(event),
            X11Event::Resized {
                new_size,
                window_id,
            } => {
                println!("Resized!");
            }
            X11Event::PresentCompleted { window_id } => {}
            X11Event::CloseRequested { window_id } => {
                signal.stop();
            }
        })
        .unwrap();

    X11BackendData {
        window,
        surface,
        renderer,
        damage_tracked_renderer,
        output,
    }
}

impl Backend for X11BackendData {
    fn init(
        event_loop: &mut EventLoop<CalloopData<Self>>,
        display: &mut Display<State<Self>>,
        space: &mut Space<Window>,
    ) -> Self {
        init_x11(event_loop, display, space)
    }

    fn draw(data: &mut CalloopData<Self>) {
        x11_draw(data)
    }

    fn size(&self) -> Size<i32, Logical> {
        let s = self.window.size();
        (s.w.into(), s.h.into()).into()
    }
}
