use std::{fs, path::Path};

use crate::state;

pub fn save(state: &state::State, filepath: &Path) -> anyhow::Result<()> {
    let persisted_state = state::persistance::PersistedState::new_from_state(state);

    let output = ron::ser::to_string_pretty(&persisted_state, ron::ser::PrettyConfig::default())?;
    fs::write(filepath, output)?;

    Ok(())
}

pub fn load(filepath: &Path, block_factory: &state::BlockFactory) -> anyhow::Result<state::State> {
    let content = fs::read_to_string(filepath)?;
    let deserialized: state::persistance::PersistedState = ron::from_str(&content)?;

    Ok(deserialized.to_state(block_factory))
}

pub fn export(filepath: &Path, data: &[f64], frequency: f64) -> anyhow::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: frequency as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(filepath, spec)?;

    for sample in data {
        writer.write_sample((*sample * i16::MAX as f64) as i16)?;
    }

    writer.finalize()?;

    Ok(())
}
