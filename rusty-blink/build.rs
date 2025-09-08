use std::env;

fn main() {
    // stm32 specific
    println!("cargo:rustc-link-arg=-Tmemory.x");
}