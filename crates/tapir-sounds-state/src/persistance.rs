use std::rc::Rc;

use serde::{Deserialize, Serialize};

use super::blocks::BlockName;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PersistedBlock {
    id: uuid::Uuid,
    name: BlockName,
    inputs: Rc<[super::Input]>,
    x: f32,
    y: f32,
}

impl PersistedBlock {
    fn new_from_block(block: &super::Block) -> Self {
        let block_pos = block.pos();

        Self {
            id: block.id().0,
            name: block.name(),
            inputs: block
                .inputs()
                .iter()
                .map(|(_, input)| input.clone())
                .collect(),
            x: block_pos.0,
            y: block_pos.1,
        }
    }

    fn to_block(&self, block_factory: &super::BlockFactory) -> super::Block {
        let mut block =
            block_factory.make_block_with_id(&self.name, (self.x, self.y), super::Id(self.id));
        for (index, input) in self.inputs.iter().enumerate() {
            block.set_input(index, input);
        }

        block
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedState {
    blocks: Rc<[PersistedBlock]>,
    connections: Rc<[(uuid::Uuid, uuid::Uuid, usize)]>,
    frequency: f64,
    selected_block: Option<uuid::Uuid>,
    should_loop: bool,
}

impl PersistedState {
    pub fn new_from_state(state: &super::State) -> Self {
        let mut blocks: Vec<_> = state.blocks().map(PersistedBlock::new_from_block).collect();
        blocks.sort_by(|a, b| a.id.cmp(&b.id)); // make the ordering consistent

        let mut connections: Vec<_> = state
            .connections()
            .map(|(output, (input, index))| (output.0, input.0, index))
            .collect();
        connections.sort();

        Self {
            blocks: blocks.into(),
            connections: connections.into(),
            frequency: state.frequency,
            selected_block: state.selected_block().map(|id| id.0),
            should_loop: state.should_loop(),
        }
    }

    pub fn to_state(&self, block_factory: &super::BlockFactory) -> super::State {
        let mut result = super::State::default();

        for block in self.blocks.iter() {
            result.add_block(block.to_block(block_factory));
        }

        for (output_id, input_id, index) in self.connections.iter() {
            result.add_connection((super::Id(*output_id), (super::Id(*input_id), *index)));
        }

        if let Some(selected_block) = self.selected_block {
            result.set_selected_block(super::Id(selected_block));
        }

        result.set_should_loop(self.should_loop);
        result.set_frequency(self.frequency);

        result
    }
}
