#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        println!("[\x1b[1;32mI\x1b[0m] {}", format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! abort {
    ($($arg:tt)*) => {{
        eprintln!("[\x1b[1;31mE\x1b[0m] {}", format_args!($($arg)*));
        std::process::exit(1);
    }};
}
