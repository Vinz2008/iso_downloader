macro_rules! print_debug {
    ($is_debug:expr_2021, $format_expr:expr_2021, $($arg:tt)*) => { if $is_debug { println!("[DEBUG]: {}", format!($format_expr, $($arg)*)) } };
}

pub(crate) use print_debug;
