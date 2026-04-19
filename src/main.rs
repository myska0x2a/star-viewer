use env_logger::Env;
use log::info;
use env_logger::WriteStyle;
use sdl3::event::*;
use stars_game::core::GameCore;
use stars_game::event::*;
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
            renderer.handle_ui_event(&event)?;

            match event {
                Event::Quit { .. } => {
                    info!("Game closing.");
                    renderer.close()?;
                    break 'main;
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

        renderer.render(&mut sdl, &resources, gameui.render_ui(&ev_sendable))?;
    }

    Ok(())
}
