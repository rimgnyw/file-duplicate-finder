use scanner::Scanner;
use surveyor::should_use_parallelism;

mod scanner;
mod surveyor;

const BASE_DIR: &str = "./test_root"; // the base directory used for testing

fn main() {
    // if let Ok(enable_parallelism) = use_parallelism() {
    //     println!("{}", enable_parallelism);
    // }
    match should_use_parallelism(BASE_DIR) {
        Ok(with_parallelism) => {
            println!("{}", with_parallelism);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    let mut scanner = Scanner::new();
    match scanner.run_scan(BASE_DIR) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
