use surveyor::use_parallelism;

mod surveyor;

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
