//! Generation entry point.
//!
//! Usage: `generate [seed] [--svg]`. With `--svg`, the SVG goes to stdout
//! (redirect it to a file) and the seed to stderr.

use confiturejeu::level::generation::gen_config::GenConfig;
use confiturejeu::level::generation::generate;
use confiturejeu::level::svg::level_to_svg;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let svg = args.iter().any(|arg| arg == "--svg");
    let seed = args
        .iter()
        .find_map(|arg| arg.parse().ok())
        .unwrap_or_else(rand::random::<u64>);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let level = generate(&mut rng, &GenConfig::default());
    if svg {
        eprintln!("seed: {seed}");
        println!("{}", level_to_svg(&level));
    } else {
        println!("seed: {seed}");
        println!("{level:?}");
    }
}
