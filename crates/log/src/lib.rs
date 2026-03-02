#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        println!("[\x1b[32mINFO \x1b[0m] {}", format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! abort {
    ($($arg:tt)*) => {{
        eprintln!("[\x1b[31mERROR\x1b[0m] {}", format_args!($($arg)*));
        std::process::exit(1);
    }};
}
