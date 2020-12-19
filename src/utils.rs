use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

const ID_SIZE: usize = 8;

pub fn random_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(ID_SIZE)
        .collect()
}
