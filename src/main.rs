// #[allow(unused)]

use game::core::GameCore;
use game::event::*;
use game::rendering::*;
use game::ui::*;
use sdl3::event::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sdl = sdl3::init()?;

    let mut gamecore = GameCore::new()?;
    let mut renderer = GameRenderer::init(&sdl)?;
    let mut gameui = GameUi::new();

    gamecore.load()?;

    let ev = sdl.event()?;
    ev.register_custom_event::<GameEvent>()?;
    let ev_sendable = ev.event_sender();

    'main: loop {
        for event in sdl.event_pump()?.poll_iter() {
            renderer.handle_ui_event(&event);

            match event {
                Event::Quit { .. } => { break 'main },
                Event::User { .. } => { 
                    let event_user = event.as_user_event_type::<GameEvent>().unwrap();
                    gamecore.handle_game_event(event_user.clone())?;
                    gameui.handle_game_event(event_user.clone())?;
                },
                _ => {},
            }
        }
        

        renderer.render(&mut sdl, gameui.render_ui(&ev_sendable))?; 
    }

    Ok(())
}
