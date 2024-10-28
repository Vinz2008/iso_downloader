macro_rules! print_debug {
    ($is_debug:expr, $format_expr:expr, $($arg:tt)*) => { if $is_debug { println!("[DEBUG]: {}", format!($format_expr, $($arg)*)) } };
}

pub(crate) use print_debug;
