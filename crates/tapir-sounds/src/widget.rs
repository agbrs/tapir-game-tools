use eframe::egui;

use crate::state;

mod block;
mod cable_state;
mod cables;
mod input;
mod port;

pub use block::block;
pub use cable_state::{CableState, PortId};
pub use cables::cables;
pub use input::input;
pub use port::{port, PortDirection};

impl From<state::Id> for egui::Id {
    fn from(val: state::Id) -> Self {
        egui::Id::new(val)
    }
}