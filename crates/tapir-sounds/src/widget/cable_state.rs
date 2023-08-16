use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use eframe::egui;

use crate::state;

#[derive(Clone, Debug, Default)]
pub struct CableState {
    inner: Arc<Mutex<CableStateInner>>,
}

impl CableState {
    pub fn from_ctx<F, T>(ctx: &egui::Context, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        ctx.data_mut(|data| f(data.get_temp_mut_or_default::<Self>(egui::Id::null())))
    }

    pub fn set_port_position(&mut self, port_id: &PortId, position: egui::Pos2) {
        self.inner
            .lock()
            .unwrap()
            .port_positions
            .insert(port_id.clone(), position);
    }

    pub fn get_port_position(&self, port_id: &PortId) -> Option<egui::Pos2> {
        self.inner
            .lock()
            .unwrap()
            .port_positions
            .get(port_id)
            .copied()
    }

    pub fn set_in_progress_cable(&mut self, port_id: &PortId) {
        self.inner
            .lock()
            .unwrap()
            .in_progress_cable
            .get_or_insert(port_id.clone());
    }

    pub fn clear_in_progress_cable(&mut self) {
        self.inner.lock().unwrap().in_progress_cable = None;
    }

    pub fn in_progress_cable(&self) -> Option<(egui::Pos2, PortId)> {
        let inner = self.inner.lock().unwrap();
        let in_progress_cable = inner.in_progress_cable.as_ref()?;

        let pos = inner.port_positions.get(in_progress_cable)?;

        Some((*pos, in_progress_cable.clone()))
    }

    pub fn closest_port_at_pos(&self, pos: egui::Pos2) -> Option<(PortId, egui::Pos2)> {
        let inner = self.inner.lock().unwrap();

        let (port, pos) = inner.port_positions.iter().min_by(|a, b| {
            a.1.distance_sq(pos)
                .partial_cmp(&b.1.distance_sq(pos))
                .expect("Distance is NaN")
        })?;

        Some((port.clone(), *pos))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub struct PortId {
    pub block_id: state::Id,
    pub index: usize,
    pub direction: super::PortDirection,
}

impl PortId {
    pub fn new(block_id: state::Id, index: usize, direction: super::PortDirection) -> Self {
        Self {
            block_id,
            index,
            direction,
        }
    }
}

#[derive(Debug, Default)]
struct CableStateInner {
    port_positions: HashMap<PortId, egui::Pos2>,
    in_progress_cable: Option<PortId>,
}