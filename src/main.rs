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

    let ev = sdl.event()?;
    let ev_sendable = ev.event_sender();

    'main: loop {
        for event in sdl.event_pump()?.poll_iter() {
            renderer.imgui.handle_event(&event);

            match event {
                Event::Quit { .. } => {
                    info!("App closing.");
                    renderer.close()?;
                    break 'main;
                }

                Event::KeyDown { keycode, .. } => {
                    let mouse = sdl.mouse();

                    if keycode == Some(Keycode::Return) {
                        renderer.mousefocus = true;
                        mouse.set_relative_mouse_mode(&renderer.window, true);
                    }
                    if keycode == Some(Keycode::Escape) {
                        renderer.mousefocus = false;
                        mouse.set_relative_mouse_mode(&renderer.window, false);
                    }
                }
                _ => {}
            }
        }

        let ui_callback = appui.build_ui();

        renderer.render(
            &mut sdl,
            &resources,
            &appcore.camera.clone(),
            ui_callback,
        )?;
    }

    Ok(())
}
