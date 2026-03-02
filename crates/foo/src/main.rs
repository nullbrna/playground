use log::abort;
use log::info;

fn main() {
    info!("Hello, info!");
    abort!("Hello, abort!");
}
