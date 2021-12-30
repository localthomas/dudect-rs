use core::arch::asm;
use std::cmp::Ordering;

use crate::statistics::TTest;

const ENOUGH_MEASUREMENTS: usize = 10000;
const NUMBER_PERCENTILES: usize = 100;
const TTEST_FAILED_MODERATE: f64 = 10.0; // test failed. Pankaj likes 4.5 but let's be more lenient
const TTEST_FAILED_OVERWHELMINGLY: f64 = 500.0;

/// Each function that should be tested must implement this trait.
pub trait MeasurementSpecimen<const N: usize> {
    /// Prepares the input data for the computation function.
    /// The input_data slice should be modified accordingly and the `is_group_a` slice has the same length.
    /// It is recommended to generate different input_data for group a and b.
    fn prepare_input_data(input_data: &mut [[u8; N]], is_group_a: &[bool]);
    /// The computation function that is analyzed for static execution time.
    fn do_one_computation(input: [u8; N]);
}

/// A context holds all the necessary information for creating and executing a measurement run.
pub struct MeasurementContext<T: MeasurementSpecimen<N>, const N: usize> {
    _specimen: T,
    /// The first tick before the first computation of a measurement run was executed.
    first_tick: u64,
    ticks: Vec<u64>,
    number_of_computations_per_run: usize,
    execution_times: Vec<u64>,
    first_order_uncropped_test: TTest,
    percentile_tests: [TTest; NUMBER_PERCENTILES],
    second_order_test: TTest,
    input_data: Vec<[u8; N]>,
    is_group_a: Vec<bool>,
    percentiles: [u64; NUMBER_PERCENTILES],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeasurementRunResult {
    LeakageFound,
    NoLeakageEvidenceYet,
}

impl<T: MeasurementSpecimen<N>, const N: usize> MeasurementContext<T, N> {
    /// Create a new measurement context with the provided data.
    pub fn new(specimen: T, number_of_computations_per_run: usize) -> Self {
        Self {
            _specimen: specimen,
            first_tick: 0,
            ticks: vec![0; number_of_computations_per_run],
            number_of_computations_per_run,
            execution_times: vec![0; number_of_computations_per_run],
            first_order_uncropped_test: TTest::new(),
            percentile_tests: [TTest::new(); NUMBER_PERCENTILES],
            second_order_test: TTest::new(),
            input_data: vec![[0u8; N]; number_of_computations_per_run],
            is_group_a: vec![false; number_of_computations_per_run],
            percentiles: [0u64; NUMBER_PERCENTILES],
        }
    }

    /// Executes a measurement run and gives back a result wether or not more runs are required.
    pub fn execute_measurement_run(&mut self) -> MeasurementRunResult {
        // randomize is_group_a
        for i in &mut self.is_group_a {
            *i = rand::random::<bool>();
        }

        T::prepare_input_data(&mut self.input_data, &self.is_group_a);
        self.measure();

        let first_time = self.percentiles[self.percentiles.len() - 1] == 0;
        if first_time {
            // throw away the first batch of measurements.
            // this helps warming things up.
            self.prepare_percentiles();
            MeasurementRunResult::NoLeakageEvidenceYet
        } else {
            self.update_statistics();
            self.report()
        }
    }

    fn measure(&mut self) {
        self.first_tick = cpu_ticks();
        for i in 0..self.number_of_computations_per_run {
            T::do_one_computation(self.input_data[i]);
            self.ticks[i] = cpu_ticks();
        }
        for i in 0..self.ticks.len() {
            let previous_tick = if i == 0 {
                self.first_tick
            } else {
                self.ticks[i - 1]
            };
            let current_tick = self.ticks[i];
            // Note: wrapping might occur when the CPU counter overflows
            self.execution_times[i] = current_tick - previous_tick;
        }
    }

    /// Prepare the percentiles with the values of the execution times as a baseline.
    /// From dudect:
    /// set different thresholds for cropping measurements.
    /// the exponential tendency is meant to approximately match
    /// the measurements distribution, but there's not more science
    /// than that.
    fn prepare_percentiles(&mut self) {
        for i in 0..self.percentiles.len() {
            self.percentiles[i] = percentile(
                &mut self.execution_times,
                1.0 - (f64::powf(0.5, 10.0 * (i as f64 + 1.0) / self.percentiles.len() as f64)),
            );
        }
    }

    fn update_statistics(&mut self) {
        // discard the first few measurements
        for i in 10..self.number_of_computations_per_run - 1 {
            let difference = self.execution_times[i] as f64;

            // t-test on the execution time
            self.first_order_uncropped_test
                .push(difference, self.is_group_a[i]);

            // t-test on cropped execution times, for several cropping thresholds
            for crop_index in 0..self.percentiles.len() {
                if difference < self.percentiles[crop_index] as f64 {
                    self.percentile_tests[crop_index].push(difference, self.is_group_a[i]);
                }
            }

            // second-order test (only if we have more than 10000 measurements).
            // Centered product pre-processing.
            if self.percentile_tests[0].get_number_of_samples()[0] > 10000.0 {
                let group_index = if self.is_group_a[i] { 0 } else { 1 };
                let centered = difference - self.percentile_tests[0].get_mean()[group_index];
                self.second_order_test
                    .push(centered * centered, self.is_group_a[i]);
            }
        }
    }

    fn report(&mut self) -> MeasurementRunResult {
        let t = self.max_test();
        let max_t = f64::abs(t.compute().unwrap_or(0.0));
        let number_traces_max_t = {
            let n = t.get_number_of_samples();
            n[0] + n[1]
        };
        let max_tau = max_t / f64::sqrt(number_traces_max_t);

        // print the number of measurements of the test that yielded max t.
        // sometimes you can see this number go down - this can be confusing
        // but can happen (different test)
        print!("meas: {:>7.2} M, ", (number_traces_max_t / 1e6));
        if number_traces_max_t < ENOUGH_MEASUREMENTS as f64 {
            println!(
                "not enough measurements ({} still to go).",
                ENOUGH_MEASUREMENTS - (number_traces_max_t as usize)
            );
            return MeasurementRunResult::NoLeakageEvidenceYet;
        }

        /*
         * We report the following statistics:
         *
         * max_t: the t value
         * max_tau: a t value normalized by sqrt(number of measurements).
         *          this way we can compare max_tau taken with different
         *          number of measurements. This is sort of "distance
         *          between distributions", independent of number of
         *          measurements.
         * (5/tau)^2: how many measurements we would need to barely
         *            detect the leak, if present. "barely detect the
         *            leak" here means have a t value greater than 5.
         *
         * The first metric is standard; the other two aren't (but
         * pretty sensible imho)
         */

        print!(
            "max t: {:>7.2}, max tau: {:.2e}, (5/tau)^2: {:.2e}.",
            max_t,
            max_tau,
            (5.0 * 5.0) / (max_tau * max_tau)
        );
        if max_t > TTEST_FAILED_OVERWHELMINGLY {
            println!(" Definitely not constant time.");
            return MeasurementRunResult::LeakageFound;
        }
        if max_t > TTEST_FAILED_MODERATE {
            println!(" Probably not constant time.");
            return MeasurementRunResult::LeakageFound;
        } else {
            println!(" For the moment, maybe constant time.");
        }
        MeasurementRunResult::NoLeakageEvidenceYet
    }

    /// Find the t-test with the maximum t value of `self.first_order_uncropped_test`, `self.percentile_tests`, and `self.second_order_test`.
    fn max_test(&self) -> TTest {
        fn max_test_function(a: &&TTest, b: &&TTest) -> Ordering {
            let a_value = a.compute().unwrap_or(0.0);
            let b_value = b.compute().unwrap_or(0.0);
            f64::partial_cmp(&a_value, &b_value).unwrap()
        }

        let mut max_test = *self
            .percentile_tests
            .iter()
            .max_by(max_test_function)
            .unwrap();
        if max_test_function(&&max_test, &&self.first_order_uncropped_test) == Ordering::Less {
            max_test = self.first_order_uncropped_test;
        }
        if max_test_function(&&max_test, &&self.second_order_test) == Ordering::Less {
            max_test = self.second_order_test;
        }
        max_test
    }
}

fn percentile(data: &mut [u64], which: f64) -> u64 {
    // it is not important for the sorting to keep the order of equal elements
    data.sort_unstable();

    let array_position = (data.len() as f64 * which) as usize;
    assert!(array_position < data.len());
    data[array_position]
}

/// Executes a function for testing and runs as long as required.
pub fn run_dudect_test<T: MeasurementSpecimen<N>, const N: usize>(specimen: T) {
    let mut dudect = MeasurementContext::new(specimen, 500);
    let mut result = MeasurementRunResult::NoLeakageEvidenceYet;
    while result == MeasurementRunResult::NoLeakageEvidenceYet {
        result = dudect.execute_measurement_run();
    }
}

/// Returns the current CPU ticks count. From the dudect implementation:
/// Intel actually recommends calling CPUID to serialize the execution flow
/// and reduce variance in measurement due to out-of-order execution.
/// We don't do that here yet.
/// see §3.2.1 http://www.intel.com/content/www/us/en/embedded/training/ia-32-ia-64-benchmark-code-execution-paper.html
pub fn cpu_ticks() -> u64 {
    let upper: u64;
    let lower: u64;
    unsafe {
        asm!("rdtsc", out("rax") lower, out("rdx") upper);
    }
    upper << 32 | lower
}
