use std::env;

mod realis_to_bsc;

fn main() {
    // Get command lines arguments
    let args: Vec<String> = env::args().collect();
    // Get command line first argument
    let arg = args.get(1);

    match arg {
        None => println!("Specify flag (realis-to-bsc or bsc-to-realis)"),
        Some(value) => {
            match value.as_str() {
                "realis-to-bsc" => realis_to_bsc::run(),
                "bsc-to-realis" => {

                }
                _ => println!("Unknown command!")
            }
        }
    }
}
