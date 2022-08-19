extern crate rsa;
extern crate crypto;
extern crate base64;


mod wx_pay;
pub use crate::wx_pay::*;

mod random;
use random::rand_string;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
