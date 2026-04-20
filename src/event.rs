//! Game event heirarchy.
use sdl3::EventPump;
use std::sync::mpsc::TryIter;
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug, Clone)]
pub enum GameEvent {
    StarSelected(String),
    StarRangeChanged(f32, usize),
    PositionChanged(f32),
    Reload,
}
