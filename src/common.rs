use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn rand_generate_num() -> String {
    let rand_string: String = thread_rng()
    .sample_iter(&Alphanumeric)
    .take(30)
    .map(char::from)
    .collect();

    rand_string
}