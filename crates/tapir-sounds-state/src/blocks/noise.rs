use std::{borrow::Cow, rc::Rc, sync::Arc};

use super::{BlockName, BlockType};

#[derive(Clone)]
pub struct Noise {
    base_amplitude: f64,
    time: f64,
    seed: f64,
}

impl Noise {
    pub fn name() -> BlockName {
        BlockName {
            category: super::BlockCategory::Fundamental,
            name: "Noise".to_owned(),
        }
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            base_amplitude: 0.5,
            time: 1.0,
            seed: Default::default(),
        }
    }
}

impl BlockType for Noise {
    fn name(&self) -> BlockName {
        Self::name()
    }

    fn inputs(&self) -> Rc<[(Cow<'static, str>, super::Input)]> {
        vec![
            ("Time".into(), super::Input::Periods(self.time)),
            (
                "Amplitude".into(),
                super::Input::Amplitude(self.base_amplitude),
            ),
            ("Seed".into(), super::Input::Periods(self.seed)),
        ]
        .into()
    }

    fn set_input(&mut self, index: usize, value: &super::Input) {
        match (index, value) {
            (0, super::Input::Periods(new_time)) => {
                if *new_time != 0.0 {
                    self.time = new_time.clamp(0.0, 5.0);
                }
            }
            (1, super::Input::Amplitude(new_amplitude)) => {
                self.base_amplitude = *new_amplitude;
            }
            (2, super::Input::Periods(new_seed)) => {
                self.seed = *new_seed;
            }
            _ => panic!("Invalid input {index} {value:?}"),
        }
    }

    fn calculate(&self, global_frequency: f64, inputs: &[Option<Arc<[f64]>>]) -> Arc<[f64]> {
        let mut rng = fastrand::Rng::with_seed(self.seed.to_bits());
        let amplitude = inputs[1].clone().unwrap_or(Arc::new([1.0]));

        let length = (self.time * global_frequency) as usize;

        let mut ret = Vec::with_capacity(length);

        for i in 0..length {
            ret.push(
                (rng.f64() * 2.0 - 1.0) * self.base_amplitude * amplitude[i % amplitude.len()],
            );
        }

        ret.into()
    }
}
