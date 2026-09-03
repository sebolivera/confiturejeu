//! Generation entry point.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use confiturejeu::level::generation::gen_config::GenConfig;
use confiturejeu::level::generation::generate;

fn main() {
    let seed = std::env::args()
    .nth(1)
    .and_then(|s| s.parse().ok())
    .unwrap_or_else(rand::random::<u64>);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let level = generate(&mut rng, &GenConfig::default());
    println!("seed: {seed}");
    println!("{level:?}");
}
