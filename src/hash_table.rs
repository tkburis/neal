use std::fmt::Debug;

use crate::{value::Value, error::ErrorType};

const INITIAL_CAPACITY: usize = 16;
const PRIME: usize = 53;
const MAX_CAPACITY: usize = 65536;
const MAX_CALC: usize = 65381;  // prime, so numbers do not divide evenly. prevents overflow in intermediate calculations
const LOAD_FACTOR_NUMERATOR: usize = 3;
const LOAD_FACTOR_DENOMINATOR: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct KeyValue<T> {
    pub key: T,
    pub value: T,
}

#[derive(Clone)]
pub struct HashTable {
    array: Vec<Vec<KeyValue<Value>>>,
    entries: usize,
    current_capacity: usize,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            array: vec![Vec::new(); INITIAL_CAPACITY],
            entries: 0,
            current_capacity: INITIAL_CAPACITY,
        }
    }

    pub fn get(&self, key: &Value, line: usize) -> Result<Value, ErrorType> {
        let bucket_number = self.get_bucket_number(&key, line)?;
        if let Some(key_value) = self.array[bucket_number].iter().find(|key_value| key_value.key == key.clone()) {
            Ok(key_value.value.clone())
        } else {
            Err(ErrorType::KeyError { key: key.clone(), line })
        }
    }

    /// Inserts a key-value pair to the table if the key does not already exist; otherwise, update the existing pair with the new value.
    pub fn insert(&mut self, key: &Value, value: &Value, line: usize) -> Result<(), ErrorType> {
        let bucket_number = self.get_bucket_number(&key, line)?;
        println!("ASSIGNED: {}", bucket_number);
        if let Some(key_value) = self.array[bucket_number].iter_mut().find(|key_value| key_value.key == key.clone()) {
            key_value.value = value.clone();
        } else {
            self.entries += 1;
            self.array[bucket_number].push(KeyValue { key: key.clone(), value: value.clone() });
        }

        self.check_load();
        Ok(())
    }

    pub fn remove(&mut self, key: &Value, line: usize) -> Result<(), ErrorType> {
        let bucket_number = self.get_bucket_number(&key, line)?;
        if let Some(index) = self.array[bucket_number].iter().position(|key_value| key_value.key == key.clone()) {
            self.array[bucket_number].remove(index);
            self.entries -= 1;
            Ok(())
        } else {
            Err(ErrorType::KeyError { key: key.clone(), line })
        }
    }

    fn check_load(&mut self) {
        if self.current_capacity < MAX_CAPACITY {
            if self.entries * LOAD_FACTOR_DENOMINATOR > self.current_capacity * LOAD_FACTOR_NUMERATOR {
                self.array.append(&mut vec![Vec::new(); self.current_capacity]);
                self.current_capacity <<= 1;
            }
        }
    }

    fn get_bucket_number(&self, key: &Value, line: usize) -> Result<usize, ErrorType> {
        // println!("{} {} -> {}", hash(key, line)?, self.current_capacity, hash(key, line)? % self.current_capacity);
        Ok(hash(key, line)? % self.current_capacity)
    }

    fn hash_self(&self, line: usize) -> Result<usize, ErrorType> {
        let mut hash_value: usize = 0;
        for bucket in self.array.iter() {
            for key_value in bucket.iter() {
                let key_hash = hash(&key_value.key, line)? % MAX_CALC;
                let value_hash = hash(&key_value.value, line)? % MAX_CALC;
                hash_value = (hash_value + key_hash * (value_hash + PRIME)) % MAX_CALC;
            }
        }
        Ok(hash_value)
    }

    pub fn flatten(&self) -> Vec<KeyValue<Value>> {
        self.array.clone().into_iter().flatten().collect()
    }
}

impl PartialEq for HashTable {
    fn eq(&self, other: &Self) -> bool {
        let self_flattened = self.flatten();
        let mut other_flattened = other.flatten();
        for self_key_value in self_flattened {
            if let Some(index) = other_flattened.iter().position(|other_key_value| *other_key_value == self_key_value) {
                other_flattened.remove(index);
            } else {
                return false;
            }
        }
        true
    }
}

impl Debug for HashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.flatten())
    }
}

fn hash(key: &Value, line: usize) -> Result<usize, ErrorType> {
    match key {
        Value::Array(array) => {
            let mut hash_value: usize = 0;
            for (index, val) in array.iter().take(20).enumerate() {  // limit to 20 so O(1)
                // http://www.cse.yorku.ca/~oz/hash.html
                let curr = (hash(val, line)? * index) % MAX_CALC;
                hash_value = (((hash_value << 5) + hash_value) + curr) % MAX_CALC;
            }
            Ok(hash_value)
        },
        Value::Bool(x) => {
            if *x {
                Ok(1)
            } else {
                Ok(2)
            }
        },
        Value::Dictionary(x) => {
            x.hash_self(line)
        },
        Value::Function { parameters: _, body: _ } => {
            Err(ErrorType::CannotHashFunction { line })
        },
        Value::Null => Ok(3),
        Value::Number(x) => {
            let mut binary: usize = (x.to_bits() >> 12).try_into().unwrap();  // to mask floating point inaccuracy
            binary %= MAX_CALC;
            binary = (binary * (binary + 3)) % MAX_CALC;  // https://www.cs.hmc.edu/~geoff/classes/hmc.cs070.200101/homework10/hashfuncs.html
            Ok(binary as usize)
        },
        Value::String_(s) => {
            let mut hash_value = 0;
            for (index, c) in s.chars().take(20).enumerate() {
                // http://www.cse.yorku.ca/~oz/hash.html
                hash_value = (((hash_value << 5) + hash_value) + (c as usize * index)) % MAX_CALC;
            }
            Ok(hash_value)
        },
    }
}
