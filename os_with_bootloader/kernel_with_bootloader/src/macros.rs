// // #[macro_export]
// // macro_rules! print {
// //         ($($stmt:tt)*) => {
// //             {
// //                 use core::fmt::write;
// //                 let frame_buffer_writer = unsafe { crate::FRAMEBUFFER_WRITER.as_mut().unwrap().as_mut() };

// //                 write!(frame_buffer_writer, $($stmt)*).unwrap();
// //             }
// //         };
// //     }

// // #[macro_export]
// // macro_rules! println {
// //         ($($stmt:tt)*) => {
// //             {
// //                 use core::fmt::write;
// //                 let frame_buffer_writer = unsafe { crate::FRAMEBUFFER_WRITER.as_mut().unwrap().as_mut() };

// //                 writeln!(frame_buffer_writer, $($stmt)*).unwrap();
// //             }
// //         };
// //     }

// #[macro_export]
// macro_rules! print {
//     ($($arg:tt)) => {
//         $crate::macros::print(core::format_args!($($arg)*));
//     };
// }
