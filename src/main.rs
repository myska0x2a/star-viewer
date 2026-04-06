use game::core::GameCore;
use game::event::*;
use game::gamepad::*;
use game::rendering::*;
use game::ui::*;

use env_logger::Env;
use log::{Level, debug, error, info, log_enabled, trace};
use sdl3::event::*;
use sdl3::gamepad::Gamepad;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start logging
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let mut sdl = sdl3::init()?;

    let mut gamecore = GameCore::new()?;
    let mut gameui = GameUi::new();

    gamecore.load()?;

    let mut renderer = GameRenderer::init(&sdl, gamecore.get_star_handler())?;

    let ev = sdl.event()?;
    ev.register_custom_event::<GameEvent>()?;
    let ev_sendable = ev.event_sender();

    let mut position: f64;

    'main: loop {
        for event in sdl.event_pump()?.poll_iter() {
            // println!("{:?}", event);
            renderer.handle_ui_event(&event);

            match event {
                Event::Quit { .. } => {
                    info!("Game closing.");
                    renderer.close()?;
                    break 'main;
                }
                Event::User { .. } => {
                    let event_user = event.as_user_event_type::<GameEvent>().unwrap();

                    // controllerhandler.handle_event(event_user.clone())?;
                    gamecore.handle_game_event(event_user.clone())?;
                    gameui.handle_game_event(event_user.clone())?;
                    renderer.handle_game_event(event_user.clone())?;
                }
                Event::MouseWheel { timestamp, window_id, which, x, y, direction, mouse_x, mouse_y } => {
                    renderer.reload_stars(gamecore.get_star_handler(), renderer.range+(x));
                    renderer.range += x;
                }
                _ => {}
            }
        }

        renderer.render(&mut sdl, gameui.render_ui(&ev_sendable))?;
    }

    Ok(())
}
