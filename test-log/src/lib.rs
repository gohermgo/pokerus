// macro_rules! expand {
//     ($($arg:tt)*) => {
//         std::fmt::format(format_args!($($arg)*))
//     };
// }
#[macro_export]
macro_rules! log {
    (tag: $tag:literal, $($arg:tt)+) => {
        println!(std::fmt::format(format_args!("[{:25}:{:4}-{:5}]{}", file!(), line!(), $tag, format!($($arg)+))))
    };
    (tag: $tag:literal) => {
        println!(std::fmt::format(format_args!("[{:25}:{:4}-{:5}]", file!(), line!(), $tag)))
    };
}
#[macro_export]
macro_rules! error {
    ($($($arg:tt)+)?) => {
        $crate::log!(tag: "ERROR"$(,$($arg)+)?);
    }
}
#[macro_export]
macro_rules! warn {
    ($($($arg:tt)+)?) => {
        $crate::log!(tag: "WARN"$(,$($arg)+)?);
    }
}
#[macro_export]
macro_rules! info {
    ($($($arg:tt)+)?) => {
        $crate::log!(tag: "INFO"$(,$($arg)+)?);
    }
}
#[macro_export]
macro_rules! debug {
    ($($($arg:tt)+)?) => {
        $crate::log!(tag: "DEBUG"$(,$($arg)+)?);
    }
}
#[macro_export]
macro_rules! trace {
    ($($($arg:tt)+)?) => {
        $crate::log!(tag: "TRACE"$(,$($arg)+)?);
    }
}
