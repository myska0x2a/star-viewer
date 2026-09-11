use env_logger::Env;
use env_logger::WriteStyle;
use log::info;
use sdl3::event::*;
use sdl3::keyboard::Keycode;
use starviewer::core::AppCore;
use starviewer::graphics::rendering::*;
use starviewer::resources::ResourceManager;
use starviewer::ui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start logging
    let logger_env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    let _logger = env_logger::Builder::from_env(logger_env)
        .format_timestamp(None)
        .write_style(WriteStyle::Auto)
        .init();

    let mut sdl = sdl3::init()?;
    let mut appcore = AppCore::new()?;
    let mut appui = AppUi::new();

    appcore.load()?;

    let mut renderer = AppRenderer::init(&sdl, appcore.get_star_handler())?;
    let resources = ResourceManager::load(&renderer.device, ".")?;

    let mut mousefocus: bool = true;
    appcore.camera.sensitivity = 0.005;

    'main: loop {
        for event in sdl.event_pump()?.poll_iter() {
            renderer.imgui.handle_event(&event);

            match event {
                Event::Quit { .. } => {
                    info!("App closing.");
                    renderer.close()?;
                    break 'main;
                }

                Event::MouseMotion {
                    timestamp,
                    window_id,
                    which,
                    mousestate,
                    x,
                    y,
                    xrel,
                    yrel,
                } => {
                    if mousefocus {
                        if xrel != 0.0f32 {
                            appcore.camera.rotate(0.0, 0.0, xrel);
                        }
                        if yrel != 0.0f32 {
                            appcore.camera.rotate(0.0, yrel, 0.0);
                        }
                    }
                }
                Event::MouseWheel { x, y, .. } => {
                    // fov control
                    if mousefocus && (y != 0.0f32) {
                        let new_fov = appcore.camera.fov + (y / 40.0);
                        appcore.camera.fov = new_fov.clamp(0.0000001, 3.14);
                    }

                    // range changing
                    if x != 0.0f32 {
                        renderer.star_renderer.range =
                            (renderer.star_renderer.range + x).clamp(0.2, 9999999.0);
                        renderer.star_renderer.reload(
                            &renderer.device,
                            &renderer.window,
                            renderer.star_renderer.range,
                        )?;
                    }
                }

                Event::KeyDown { keycode, .. } => {
                    let mouse = sdl.mouse();

                    if keycode == Some(Keycode::Return) {
                        mousefocus = true;
                        mouse.set_relative_mouse_mode(&renderer.window, true);
                    }
                    if keycode == Some(Keycode::Escape) {
                        mousefocus = false;
                        mouse.set_relative_mouse_mode(&renderer.window, false);
                    }
                }
                _ => {}
            }
        }

        renderer.render(&mut sdl, &resources, &mut appcore, &mut appui)?;
    }

    Ok(())
}
