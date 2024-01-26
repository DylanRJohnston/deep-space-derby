use rand::{distributions::Uniform, prelude::*};

pub fn generate_game_code() -> String {
    Uniform::from('A'..='Z')
        .sample_iter(&mut thread_rng())
        .take(6)
        .collect()
}
