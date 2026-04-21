use env_logger::Env;
use env_logger::WriteStyle;
use log::info;
use sdl3::event::*;
use sdl3::keyboard::Keycode;
use stars_game::core::GameCore;
use stars_game::event::*;
use stars_game::gamepad::ControllerHandler;
use stars_game::graphics::rendering::*;
use stars_game::resources::ResourceManager;
use stars_game::ui::*;

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
    let mut gamecore = GameCore::new()?;
    let mut gameui = GameUi::new();

    gamecore.load()?;

    let mut renderer = GameRenderer::init(&sdl, gamecore.get_star_handler())?;
    let resources = ResourceManager::load(&renderer.device, ".")?;

    let ev = sdl.event()?;
    ev.register_custom_event::<GameEvent>()?;
    let ev_sendable = ev.event_sender();

    'main: loop {
        for event in sdl.event_pump()?.poll_iter() {
            // this should not be handled by the renderer directly. an input handler
            // will need to be created which abstracts inputs into game events.
            renderer.handle_ui_event(&event)?;

            match event {
                Event::Quit { .. } => {
                    info!("Game closing.");
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

                Event::MouseWheel { x, y, .. } => {
                    if renderer.mousefocus {
                        let new_fov = renderer.camera.fov + (y / 10.0);
                        renderer.camera.fov = new_fov.clamp(0.0000001, 3.14);
                    }
                    if x != 0.0f32 {
                        renderer.star_renderer.range =
                            (renderer.star_renderer.range + x).clamp(0.2, 9999999.0);
                        renderer.star_renderer.reload(
                            &renderer.device,
                            &renderer.window,
                            renderer.star_renderer.range,
                        )?;

                        ev.push_custom_event(GameEvent::StarRangeChanged(renderer.star_renderer.range, renderer.star_renderer.num_stars))?;
                    }
                }

                Event::User { .. } => {
                    let event_user = event.as_user_event_type::<GameEvent>().unwrap();

                    gamecore.handle_game_event(event_user.clone())?;
                    gameui.handle_game_event(event_user.clone())?;
                    renderer.handle_game_event(event_user.clone())?;
                }
                _ => {}
            }
        }

        renderer.render(&mut sdl, &resources, gameui.render_ui(&ev_sendable, &mut gamecore))?;
    }

    Ok(())
}
