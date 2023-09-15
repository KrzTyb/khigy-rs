use crate::khigy;
use anyhow::Result;
use smithay::{
    backend::{
        renderer::{
            element::{
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                Kind,
            },
            gles::GlesRenderer,
            utils::draw_render_elements,
            Frame, Renderer,
        },
        winit::{
            self as winit_smithay, WinitError, WinitEvent, WinitEventLoop, WinitGraphicsBackend,
        },
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop,
        },
        wayland_server::protocol::wl_surface,
        winit,
    },
    utils::{Rectangle, Transform},
    wayland::compositor::{with_surface_tree_downward, SurfaceAttributes, TraversalAction},
};
use std::time::Duration;

pub fn run(event_loop: &mut EventLoop<crate::LoopData>, data: &mut crate::LoopData) -> Result<()> {
    let display = &mut data.display;

    let (mut backend, mut winit_event_loop) = winit_smithay::init_from_builder::<GlesRenderer>(
        winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0))
            .with_title("Khigy")
            .with_visible(true),
    )
    .map_err(|error| anyhow::anyhow!("Init winit window failed. Reason: {}", error.to_string()))?;

    let mode = Mode {
        size: backend.window_size().physical_size,
        refresh: 60_000,
    };

    let output = Output::new(
        "winit".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Khigy".into(),
            model: "Winit".into(),
        },
    );
    let _global = output.create_global::<khigy::KhigyState>(&display.handle());
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    let timer = Timer::immediate();
    event_loop
        .handle()
        .insert_source(timer, move |_, _, data| {
            winit_dispatch(&mut backend, &mut winit_event_loop, data, &output).unwrap();
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .map_err(|error| {
            anyhow::anyhow!(
                "Failed to insert event loop source. Reason: {}",
                error.to_string()
            )
        })?;

    Ok(())
}

pub fn winit_dispatch(
    backend: &mut WinitGraphicsBackend<GlesRenderer>,
    winit: &mut WinitEventLoop,
    data: &mut crate::LoopData,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let display = &mut data.display;
    let state = &mut data.state;

    let res = winit.dispatch_new_events(|event| match event {
        WinitEvent::Resized { size, .. } => {
            output.change_current_state(
                Some(Mode {
                    size,
                    refresh: 60_000,
                }),
                None,
                None,
                None,
            );
        }
        WinitEvent::Input(event) => state.process_input_event(event),
        _ => (),
    });

    if let Err(WinitError::WindowClosed) = res {
        // Stop the loop
        state.loop_signal.stop();

        return Ok(());
    } else {
        res?;
    }

    backend.bind()?;

    let size = backend.window_size().physical_size;
    let damage = Rectangle::from_loc_and_size((0, 0), size);

    let elements = state
        .xdg_shell_state
        .toplevel_surfaces()
        .iter()
        .flat_map(|surface| {
            render_elements_from_surface_tree(
                backend.renderer(),
                surface.wl_surface(),
                (0, 0),
                1.0,
                1.0,
                Kind::Unspecified,
            )
        })
        .collect::<Vec<WaylandSurfaceRenderElement<GlesRenderer>>>();

    let mut frame = backend
        .renderer()
        .render(size, Transform::Flipped180)
        .unwrap();
    frame.clear([0.0, 0.0, 0.0, 1.0], &[damage]).unwrap();
    draw_render_elements(&mut frame, 1.0, &elements, &[damage]).unwrap();
    // We rely on the nested compositor to do the sync for us
    let _ = frame.finish().unwrap();

    for surface in state.xdg_shell_state.toplevel_surfaces() {
        send_frames_surface_tree(
            surface.wl_surface(),
            state.start_time.elapsed().as_millis() as u32,
        );
    }

    display.dispatch_clients(state)?;
    display.flush_clients()?;

    // It is important that all events on the display have been dispatched and flushed to clients before
    // swapping buffers because this operation may block.
    backend.submit(Some(&[damage])).unwrap();

    Ok(())
}

pub fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            // the surface may not have any user_data if it is a subsurface and has not
            // yet been committed
            for callback in states
                .cached_state
                .current::<SurfaceAttributes>()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}
