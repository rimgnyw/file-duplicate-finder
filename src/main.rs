use std::time::Instant;

use scanner::run_scan;
use surveyor::should_use_parallelism;

mod scanner;
mod surveyor;

const BASE_DIR: &str = "./test_root"; // the base directory used for testing

fn main() {
    // if let Ok(enable_parallelism) = use_parallelism() {
    //     println!("{}", enable_parallelism);
    // }

    let start = Instant::now();

    match should_use_parallelism(BASE_DIR) {
        Ok(with_parallelism) => {
            println!("{}", with_parallelism);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    match run_scan(BASE_DIR) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    let duration = start.elapsed();
    println!("Time Elapse: {:?}", duration);
}
