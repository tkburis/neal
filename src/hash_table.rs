// use crate::{token::Value, error::ErrorType, expr::KeyValue};

// const INITIAL_CAPACITY: usize = 16;
// const PRIME: usize = 53;

// pub struct HashTable {
//     array: Vec<KeyValue<Value>>,
//     entries: usize,
//     current_capacity: usize,
// }

// impl HashTable {
//     pub fn new() -> Self {
//         Self {
//             array: Vec::with_capacity(INITIAL_CAPACITY),
//             entries: 0,
//             current_capacity: INITIAL_CAPACITY,
//         }
//     }

//     pub fn insert(&self, key: Value, value: Value, line: usize) -> Result<(), ErrorType> {
//     }
    
//     fn hash(&self, key: &Value) -> usize {
//         match key {
//             Value::Array(array) => {
//                 let mut hash_value: usize = 0;
//                 let mut prime_pow: usize = 1;
//                 for x in array.iter() {
//                     hash_value = (hash_value + self.hash(x) * prime_pow) % self.current_capacity;
//                     prime_pow = (prime_pow * PRIME) % self.current_capacity;
//                 }
//                 hash_value
//             },
//             Value::Bool(x) => {
//                 if *x {
//                     1
//                 } else {
//                     2
//                 }
//             }
//             Value::Dictionary(x) => {},
//             Value::Null => 3,
//             Value::Number(x) => {},
//             Value::String_(x) => {},
//         }
//     }
// }
// ! problem: O(1) hash func?