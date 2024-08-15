// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

#![deny(missing_docs)]
//! A simple key/value store.
pub use kv::KvStore;
/// A simple key/value store.
pub type Result<T> = anyhow::Result<T>;

mod kv;

