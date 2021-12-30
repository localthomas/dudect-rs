mod dudect;
mod statistics;

use dudect::{run_dudect_test, MeasurementSpecimen};
use rand::RngCore;

fn main() {
    run_dudect_test(ThreadSleep {});
}

struct ThreadSleep {}

impl MeasurementSpecimen<1> for ThreadSleep {
    fn prepare_input_data(input_data: &mut [[u8; 1]], _is_group_a: &[bool]) {
        for input_data in input_data {
            // Group A and B contain random bytes, which means they do not differ when executed
            rand::thread_rng().fill_bytes(input_data);
        }
    }

    fn do_one_computation(input: [u8; 1]) {
        // sleep for the length of x microseconds
        let sleep_micros = input[0];
        std::thread::sleep(std::time::Duration::from_micros(sleep_micros as u64));
    }
}
