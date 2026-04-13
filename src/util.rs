//! WIP service locator, may be scrapped.
use sdl3::Sdl;

pub struct GameUtils {
    sdl: Sdl,
}

impl GameUtils {
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let mut sdl = sdl3::init()?;
        return Ok(GameUtils { sdl });
    }
}
