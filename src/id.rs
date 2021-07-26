/*
 * Generates a unique id by converting the current time
 * into a string and the converting that string into utf8
 * bytes and multiplying the bytes by a random floating 64
 * number
 */
use rand::prelude::*;

pub type Id = f64;

pub fn gen_id() -> Id {
    multiply_contents_by_a_random_number(
        chrono::offset::Local::now()
            .to_string()
            .as_bytes()
            .to_owned(),
    )
}

fn multiply_contents_by_a_random_number(iter: Vec<u8>) -> f64 {
    let mut result: f64 = 0.0;
    let mut rng = rand::thread_rng();
    for item in iter {
        result += item as f64 * rng.gen::<f64>();
    }
    result
}

#[test]
fn test_gen_id() {
    let mut previous: f64 = 0.0;
    for _ in 0..1000 {
        let current: f64 = gen_id();
        if previous == current {
            assert!(false);
        }
        previous = current;
    }
    assert!(true);
}
