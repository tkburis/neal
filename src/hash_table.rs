use std::fmt::Debug;

use crate::{value::Value, error::ErrorType};

// Hash table constants.
const INITIAL_CAPACITY: usize = 16;  // Initial number of buckets in the table.
const MAX_CAPACITY: usize = 65536;  // The maximum number of buckets in the table.
const MAX_CALC: usize = 65381;  // A prime used to prevent overflow in intermediate calculations.
const LOAD_FACTOR_NUMERATOR: usize = 3;  // Numerator of the maximum load factor before a rehash is required (3/4).
const LOAD_FACTOR_DENOMINATOR: usize = 4;  // Denominator of the maximum load factor before a rehash is required (3/4).
const HASH_FIRST_N: usize = 300;  // Number of elements to hash to keep constant time operation.

// A key-value pair in the hash table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyValue<T> {
    pub key: T,
    pub value: T,
}

/// A hash table.
#[derive(Clone)]
pub struct HashTable {
    array: Vec<Vec<KeyValue<Value>>>,  // The internal array of the hash table.
    entries: usize, // The number of entries in the hash table.
    current_capacity: usize,  // The current capacity in the hash table, i.e., the current number of buckets.
}

impl HashTable {
    /// Initialises a new instance of `HashTable`.
    pub fn new() -> Self {
        Self {
            array: vec![Vec::new(); INITIAL_CAPACITY],  // Initially has an `INITIAL_CAPACITY` number of buckets.
            entries: 0,
            current_capacity: INITIAL_CAPACITY,
        }
    }

    /// Returns the value associated with `key`.
    pub fn get(&self, key: &Value, line: usize) -> Result<&Value, ErrorType> {
        // Calculate the bucket number of the key.
        let bucket_number = self.get_bucket_number(key, line)?;

        // Iterate through the bucket.
        if let Some(key_value) = self.array[bucket_number].iter().find(|key_value| key_value.key == key.clone()) {
            // If a `key_value` is found such that `key_value.key == key`, then return `key_value.value`.
            Ok(&key_value.value)
        } else {
            // Otherwise, the key does not exist in the table. Return a KeyError, providing the `key` for detail.
            Err(ErrorType::KeyError { key: key.clone(), line })
        }
    }

    /// Returns a mutable reference to the value associated with `key`. As above.
    pub fn get_mut(&mut self, key: &Value, line: usize) -> Result<&mut Value, ErrorType> {
        let bucket_number = self.get_bucket_number(key, line)?;
        if let Some(key_value) = self.array[bucket_number].iter_mut().find(|key_value| key_value.key == key.clone()) {
            Ok(&mut key_value.value)
        } else {
            Err(ErrorType::KeyError { key: key.clone(), line })
        }
    }

    /// Inserts a key-value pair to the table if the key does not already exist; otherwise, updates the existing pair with the new value.
    pub fn insert(&mut self, key: &Value, value: &Value, line: usize) -> Result<(), ErrorType> {
        // Calculate the bucket number of the key.
        let bucket_number = self.get_bucket_number(key, line)?;

        // Iterate through the bucket.
        if let Some(key_value) = self.array[bucket_number].iter_mut().find(|key_value| key_value.key == key.clone()) {
            // If a `key_value` is found such that `key_value.key == key`, then update `key_value.value` to `value`.
            key_value.value = value.clone();
        } else {
            // Otherwise, we are adding a new entry.
            self.entries += 1;  // Increment the number of entries in the table.
            self.array[bucket_number].push(KeyValue {  // Push the new key-value pair into the bucket.
                key: key.clone(),
                value: value.clone()
            });
        }
        
        // Check if the table needs rehashing.
        self.check_load(line)?;

        Ok(())
    }

    /// Removes a key-value pair from the table.
    pub fn remove(&mut self, key: &Value, line: usize) -> Result<(), ErrorType> {
        // Calculate the bucket number of the key.
        let bucket_number = self.get_bucket_number(key, line)?;

        // Iterate through the bucket.
        if let Some(index) = self.array[bucket_number].iter().position(|key_value| key_value.key == key.clone()) {
            // If an `index` is found such that `bucket[index].key == key`, then remove the entry at that index.
            self.array[bucket_number].remove(index);
            self.entries -= 1;  // Decrement the number of entries in the table.
            Ok(())
        } else {
            // Otherwise, the key does not exist in the table. Return a KeyError, providing the `key` for detail.
            Err(ErrorType::KeyError { key: key.clone(), line })
        }
    }

    /// Returns the number of entries in the table.
    pub fn size(&self) -> usize {
        self.entries
    }

    /// Checks the load factor of the table and performs rehashing if required.
    fn check_load(&mut self, line: usize) -> Result<(), ErrorType> {
        if self.current_capacity < MAX_CAPACITY && self.entries * LOAD_FACTOR_DENOMINATOR > self.current_capacity * LOAD_FACTOR_NUMERATOR {
            // If `current_capacity` is less than the maximum capacity and greater than the maximum load factor, perform rehashing.

            // Make a copy of the entries in the table.
            let copy = self.flatten();

            // Double the current capacity of the table.
            self.current_capacity <<= 1;

            // Repopulate the internal array with `current_capacity` number of empty buckets.
            self.array = vec![Vec::new(); self.current_capacity];

            // For each entry in the saved table, re-insert it in the new table.
            for entry in copy.iter() {
                self.insert(&entry.key, &entry.value, line)?;
            }
        }
        Ok(())
    }

    /// Calculates the bucket number of a key.
    fn get_bucket_number(&self, key: &Value, line: usize) -> Result<usize, ErrorType> {
        println!("{:#?}", hash(key, HASH_FIRST_N, line)?.0);
        Ok(hash(key, HASH_FIRST_N, line)?.0 % self.current_capacity)
    }

    /// Returns all the key-value pairs in the table in a one-dimensional array.
    pub fn flatten(&self) -> Vec<KeyValue<Value>> {
        self.array.clone().into_iter().flatten().collect()
    }
}

/// Other parts of the interpreter rely on being able to compare two `Value`s.
/// Since `HashTable` will be used as as part of a `Value` variant, it has to be comparable.
/// Here, two `HashTable`s are equal if they contain the same set of key-value pairs.
impl PartialEq for HashTable {
    fn eq(&self, other: &Self) -> bool {
        // One-dimensional array of entries in `self`.
        let self_flattened = self.flatten();

        // Array of entries in `other`.
        let mut other_flattened = other.flatten();

        // If they do not contain the same number of entries, they are not equal.
        if self_flattened.len() != other_flattened.len() {
            return false;
        }

        // Iterate through the entries of `self`. If it exists in `other` as well, remove it from `other`.
        // If it does not, then the two hash tables do not contain the same entries, so we return `false`.
        for self_key_value in self_flattened {
            if let Some(index) = other_flattened.iter().position(|other_key_value| *other_key_value == self_key_value) {
                other_flattened.remove(index);
            } else {
                return false;
            }
        }

        // All entries in `self` correspond to an entry in `other`, and they have the same number of entries,
        // so they must be equal.
        true
    }
}

/// Used for printing hash tables.
impl Debug for HashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.flatten())
    }
}

/// Computes and returns the (hash, elements_left) of a key.
fn hash(key: &Value, mut elements_left: usize, line: usize) -> Result<(usize, usize), ErrorType> {
    match key {
        Value::Array(array) => {
            // The `djb2` algorithm is used. (https://theartincode.stanis.me/008-djb2/)
            let mut hash_value: usize = 5381;
            let mut index: usize = 0;

            // Limit the number of elements to `elements_left`, which can change depending on the recursive calls.
            while elements_left > 0 && index < array.len() {
                let result = hash(&array[index], elements_left, line)?;  // (hash, elements_left)
                let curr = result.0 % MAX_CALC;
                hash_value = (((hash_value << 5) + hash_value) + curr) % MAX_CALC; // Equivalent to `* 33 + curr`, but faster
                
                elements_left = result.1;
                index += 1;
            }

            Ok((hash_value, elements_left))
        },
        Value::Bool(b) => {
            if *b {
                Ok((1, elements_left - 1))
            } else {
                Ok((2, elements_left - 1))
            }
        },
        Value::Dictionary(..) => {
            // Hashing dictionaries in constant time will involve more sophisticated techniques.
            Err(ErrorType::CannotHashDictionary { line })
        },
        Value::Function {..} | Value::BuiltinFunction(..) => {
            // It is tricky to hash functions as the comparison of two functions is not set in stone.
            // So we raise a descriptive error instead.
            Err(ErrorType::CannotHashFunction { line })
        },
        Value::Null => Ok((3, elements_left - 1)),
        Value::Number(x) => {
            // We will discard the 12 least significant bits to mask floating point inaccuracy.
            let mut binary: usize = (x.to_bits() >> 12).try_into().unwrap();
            binary %= MAX_CALC;

            // The 'Knuth Variant on Division' (https://www.cs.hmc.edu/~geoff/classes/hmc.cs070.200101/homework10/hashfuncs.html)
            binary = (binary * (binary + 3)) % MAX_CALC;
            Ok((binary as usize, elements_left - 1))
        },
        Value::String_(s) => {
            // Similar to arrays, we use the `djb2` algorithm.
            let mut hash_value = 5381;
            let mut index = 0;

            while elements_left > 0 && index < s.chars().count() {
                hash_value = (((hash_value << 5) + hash_value) + s.chars().nth(index).unwrap() as usize) % MAX_CALC;
                elements_left -= 1;
                index += 1;
            }

            Ok((hash_value, elements_left))
        },
    }
}
