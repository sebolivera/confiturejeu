use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct OpeningWeights {
    /// Pre-compiled distribution for fast sampling
    dist: WeightedIndex<u32>,
}

impl OpeningWeights {
    /// Creates a new weight profile for hex rooms.
    pub fn new(weights: [u32; 6]) -> Self {
        Self {
            dist: WeightedIndex::new(&weights).expect("Sum of all weights must not be zero"),
        }
    }

    /// Samples the distribution and returns a count between 1 and 6.
    pub fn sample_count(&self, rng: &mut impl Rng) -> usize {
        self.dist.sample(rng) + 1
    }
}
