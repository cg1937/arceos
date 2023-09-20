// extern crate std;

#[macro_export]
macro_rules! size_of_struct {
    ($struct_type:ty) => {
        core::mem::size_of::<$struct_type>()
    };
}

// #[macro_export]
// macro_rules! log_info {
//     ($fmt:expr $(, $arg:expr)*) => {
//         std::println!("[{}:{}] {}", file!(), line!(), std::format!($fmt $(, $arg)*));
//     };
// }
