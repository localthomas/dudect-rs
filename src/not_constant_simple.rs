mod dudect;
mod statistics;

use dudect::{run_dudect_test, MeasurementSpecimen};
use rand::RngCore;

fn main() {
    run_dudect_test(ThreadSleep {});
}

struct ThreadSleep {}

impl MeasurementSpecimen<1> for ThreadSleep {
    fn prepare_input_data(input_data: &mut [[u8; 1]], is_group_a: &[bool]) {
        for i in 0..is_group_a.len() {
            // Group A contains random bytes; Group B only 0u8
            let is_group_a = is_group_a[i];
            if is_group_a {
                rand::thread_rng().fill_bytes(&mut input_data[i]);
            } else {
                input_data[i] = [0u8; 1];
            }
        }
    }

    fn do_one_computation(input: [u8; 1]) {
        // sleep for the length of x microseconds
        let sleep_micros = input[0];
        std::thread::sleep(std::time::Duration::from_micros(sleep_micros as u64));
    }
}
