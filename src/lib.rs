extern crate base64;
extern crate crypto;
extern crate rsa;

mod random;
mod utils;

mod wx_pay;
pub use crate::wx_pay::*;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
