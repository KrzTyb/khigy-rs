use std::time::Duration;

use anyhow::Result;
use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{WinitError, WinitEvent, WinitEventLoop, WinitGraphicsBackend},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop,
        },
        wayland_server::Display,
        winit::window::WindowBuilder,
    },
    utils::{Physical, Rectangle, Size, Transform},
};

use crate::khigy;

pub struct WinitBackend {
    graphics_backend: WinitGraphicsBackend<GlesRenderer>,
    winit_event_loop: WinitEventLoop,
    output: Output,
    damage_tracker: OutputDamageTracker,
}

pub fn create_backend(
    event_loop: &mut EventLoop<khigy::LoopData>,
    data: &mut khigy::LoopData,
) -> Result<()> {
    let window_builder = WindowBuilder::new().with_title("Khigy");

    let (winit_graphics_backend, winit) =
        smithay::backend::winit::init_from_builder::<GlesRenderer>(window_builder).map_err(
            |error| {
                anyhow::anyhow!(
                    "Failed to create winit backend. Reason: {}",
                    error.to_string()
                )
            },
        )?;

    // Create output (winit has single output)
    let output = create_output(
        &mut data.display,
        winit_graphics_backend.window_size().physical_size,
    );
    let damage_tracker = OutputDamageTracker::from_output(&output);

    data.backend = khigy::Backend::Winit(WinitBackend {
        graphics_backend: winit_graphics_backend,
        winit_event_loop: winit,
        output,
        damage_tracker,
    });

    let timer = Timer::immediate();
    event_loop
        .handle()
        .insert_source(timer, move |_, _, data| {
            if let Err(error) = process_winit(data) {
                log::error!("Failed to process winit! Reason: {}", error);
                data.state.loop_signal.stop();
                return TimeoutAction::Drop;
            }
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .map_err(|error| {
            anyhow::anyhow!(
                "Failed to insert timer source for winit dispatch. Reason: {}",
                error
            )
        })?;

    Ok(())
}

fn create_output(
    display: &mut Display<khigy::State>,
    physical_size: Size<i32, Physical>,
) -> Output {
    let output = Output::new(
        "WinitOutput".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Khigy".into(),
            model: "WinitModel".into(),
        },
    );

    let _global = output.create_global::<khigy::State>(&display.handle());

    let mode = Mode {
        size: physical_size,
        refresh: 60_000,
    };

    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    output
}

fn process_winit(data: &mut khigy::LoopData) -> Result<()> {
    let winit_backend = match &mut data.backend {
        khigy::Backend::Winit(backend) => backend,
        _ => unreachable!("Other backend in winit event loop!"),
    };

    let size = winit_backend.graphics_backend.window_size().physical_size;
    let damage = Rectangle::from_loc_and_size((0, 0), size);

    let res = winit_backend
        .winit_event_loop
        .dispatch_new_events(|event| match event {
            WinitEvent::Resized { size, .. } => {
                winit_backend.output.change_current_state(
                    Some(Mode {
                        size,
                        refresh: 60_000,
                    }),
                    None,
                    None,
                    None,
                );
            }
            WinitEvent::Input(event) => data.state.process_input_event(event),
            _ => (),
        });

    if let Err(WinitError::WindowClosed) = res {
        data.state.loop_signal.stop();
        return Ok(());
    } else {
        res?;
    }

    winit_backend.graphics_backend.bind()?;
    winit_backend.graphics_backend.submit(Some(&[damage]))?;

    data.display.flush_clients()?;
    Ok(())
}
