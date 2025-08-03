use scanner::use_parallelism;

mod scanner;

fn main() {
    // if let Ok(enable_parallelism) = use_parallelism() {
    //     println!("{}", enable_parallelism);
    // }
    match use_parallelism() {
        Ok(value) => {
            println!("{}", value);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
