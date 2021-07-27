use std::env;
use tokio;

mod bsc_to_realis;
mod realis_to_bsc;

#[tokio::main]
async fn main() {
    // Get command lines arguments
    let args: Vec<String> = env::args().collect();
    // Get command line first argument
    let arg = args.get(1);

    match arg {
        None => println!("Specify flag (realis-to-bsc or bsc-to-realis)"),
        Some(value) => match value.as_str() {
            "realis-to-bsc" => realis_to_bsc::run().await,
            "bsc-to-realis" => bsc_to_realis::run().await,
            _ => println!("Unknown command!"),
        },
    }
}
