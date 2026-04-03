use crate::stars;
use log::info;

/// An individual ship.
pub struct Ship {
    name: String,
}

impl Ship {
    pub fn new(name: String) -> Self {
        return Ship { name: name };
    }
}

/// A collection of ships heading to a star (a 'journey').
pub struct Expedition {
    ships: Vec<Ship>,
    origin: String,
    destination: String,
    journey_distance: f32,
    journey_completion: f32,
}

impl Expedition {
    pub fn new() -> Self {
        let ship = Ship::new(String::from("Leonara Christine"));
        let mut ships = Vec::new();
        ships.push(ship);

        return Expedition {
            ships,
            origin: String::from("Sol"),
            destination: String::from("Beta Three"),
            journey_distance: 22.3,
            journey_completion: 12.1,
        };
    }
}

/// Handles and updates the state of all ships in flight.
pub struct FleetHandler {
    fleet: Vec<Expedition>,
}

impl FleetHandler {
    pub fn new() -> Self {
        info!("Init fleet handler");
        let mut fleet = Vec::new();
        fleet.push(Expedition::new());
        return FleetHandler { fleet: fleet };
    }
}
