// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

//! # Accumulator
//!
//! A key-value store limited by size or time, allowing accumulation of multiple values under a
//! single key.
//!
//! When adding (accumulating) values under a given key, once a predefined quorum count has been
//! reached, the function will thereafter return all the accumulated values for that particular key.

#![doc(html_logo_url =
           "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
       html_root_url = "http://maidsafe.github.io/accumulator")]

// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(bad_style, exceeding_bitshifts, mutable_transmutes, no_mangle_const_items,
          unknown_crate_types, warnings)]
#![deny(deprecated, drop_with_repr_extern, improper_ctypes, missing_docs,
        non_shorthand_field_patterns, overflowing_literals, plugin_as_library,
        private_no_mangle_fns, private_no_mangle_statics, stable_features, unconditional_recursion,
        unknown_lints, unsafe_code, unused, unused_allocation, unused_attributes,
        unused_comparisons, unused_features, unused_parens, while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]
#![allow(box_pointers, fat_ptr_transmutes, missing_copy_implementations,
         missing_debug_implementations, variant_size_differences)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy, clippy_pedantic))]
#![cfg_attr(feature="clippy", allow(use_debug))]

// Non-MaidSafe crates
#[macro_use]
extern crate log;
#[cfg(test)]
#[macro_use]
#[allow(unused_extern_crates)]  // Only using macros from maidsafe_utilites
extern crate maidsafe_utilities;
#[cfg(test)]
extern crate rand;
extern crate time;

// MaidSafe crates
extern crate lru_time_cache;

use lru_time_cache::LruCache;
use time::Duration;

/// Implementation of [Accumulator](index.html#accumulator).
pub struct Accumulator<Key, Value>
    where Key: PartialOrd + Ord + Clone,
          Value: Clone
{
    // Expected threshold for resolve
    quorum: usize,
    lru_cache: LruCache<Key, Vec<Value>>,
}

#[cfg_attr(feature="clippy", allow(wrong_self_convention))]
impl<Key: PartialOrd + Ord + Clone, Value: Clone> Accumulator<Key, Value> {
    /// Constructor for capacity based `Accumulator`.
    ///
    /// `quorum` defines the count at and above which [`add()`](#method.add) will return `Some()`.
    pub fn with_capacity(quorum: usize, capacity: usize) -> Accumulator<Key, Value> {
        Accumulator {
            quorum: quorum,
            lru_cache: LruCache::<Key, Vec<Value>>::with_capacity(capacity),
        }
    }

    /// Constructor for time based `Accumulator`.
    ///
    /// `quorum` defines the count at and above which [`add()`](#method.add) will return `Some()`.
    pub fn with_duration(quorum: usize, duration: Duration) -> Accumulator<Key, Value> {
        Accumulator {
            quorum: quorum,
            lru_cache: LruCache::<Key, Vec<Value>>::with_expiry_duration(duration),
        }
    }

    /// Returns whether `key` exists in the accumulator or not.
    pub fn contains_key(&mut self, key: &Key) -> bool {
        self.lru_cache.contains_key(key)
    }

    /// Returns whether `key` exists and has accumulated `quorum` or more corresponding values.
    pub fn is_quorum_reached(&mut self, key: &Key) -> bool {
        match self.lru_cache.get(key) {
            None => false,
            Some(entry) => entry.len() >= self.quorum,
        }
    }

    /// Adds a key-value pair.
    ///
    /// Returns the corresponding values for `key` if `quorum` or more values have been accumulated,
    /// otherwise returns `None`.
    pub fn add(&mut self, key: Key, value: Value) -> Option<Vec<Value>> {
        if self.contains_key(&key) {
            if let Some(result) = self.lru_cache.get_mut(&key) {
                result.push(value)
            } else {
                debug!("key found cannot push to value")
            }
        } else {
            let _ = self.lru_cache.insert(key.clone(), vec![value]);
        }

        // FIXME(dirvine) This iterates too many times,
        // should combine and answer in one iteration :27/08/2015
        if self.is_quorum_reached(&key) {
            match self.lru_cache.get(&key) {
                Some(value) => Some(value.clone()),
                None => None,
            }
        } else {
            None
        }
    }

    /// Retrieves a clone of the values accumulated under `key`, or `None`  if `key` doesn't exist.
    pub fn get(&mut self, key: &Key) -> Option<Vec<Value>> {
        match self.lru_cache.get(key) {
            Some(entry) => Some(entry.clone()),
            None => None,
        }
    }

    /// Removes `key` and all corresponding accumulated values.
    pub fn delete(&mut self, key: &Key) {
        let _ = self.lru_cache.remove(key);
    }

    /// Returns the size of the accumulator, i.e. the number of keys held.
    pub fn cache_size(&mut self) -> usize {
        self.lru_cache.len()
    }

    /// Sets a new value for `quorum`.
    ///
    /// This has immediate effect, even for existing key-value entries.
    pub fn set_quorum_size(&mut self, new_size: usize) {
        self.quorum = new_size;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    #[test]
    fn add() {
        let mut accumulator = Accumulator::with_capacity(1, 100);

        assert!(accumulator.add(2, 3).is_some());
        assert_eq!(accumulator.contains_key(&1), false);
        assert_eq!(accumulator.contains_key(&2), true);
        assert_eq!(accumulator.is_quorum_reached(&1), false);
        assert_eq!(accumulator.is_quorum_reached(&2), true);
        assert!(accumulator.add(1, 3).is_some());
        assert_eq!(accumulator.contains_key(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), true);
        assert!(accumulator.add(1, 3).is_some());
        assert_eq!(accumulator.contains_key(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), true);

        let mut responses = unwrap_option!(accumulator.get(&1), "");

        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0], 3);

        responses = unwrap_option!(accumulator.get(&2), "");

        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], 3);
    }

    #[test]
    fn add_single_value_quorum() {
        let quorum_size = 19;
        let mut accumulator = Accumulator::with_capacity(quorum_size, 100);
        let key = random::<i32>();
        let value = random::<u32>();
        for i in 0..quorum_size - 1 {
            assert!(accumulator.add(key, value).is_none());
            let retrieved_value = unwrap_option!(accumulator.get(&key), "");
            assert_eq!(retrieved_value.len(), i + 1);
            // for response in value { assert_eq!(response, value); };
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, value).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
        let retrieved_value = unwrap_option!(accumulator.get(&key), "");
        assert_eq!(retrieved_value.len(), quorum_size);
        // for response in value { assert_eq!(response, value); };
    }

    #[test]
    fn add_multiple_values_quorum() {
        let quorum_size = 19;
        let mut accumulator = Accumulator::with_capacity(quorum_size, 100);
        let key = random::<i32>();
        for _ in 0..quorum_size - 1 {
            assert!(accumulator.add(key, random::<u32>()).is_none());
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, random::<u32>()).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
    }

    #[test]
    fn add_multiple_keys_quorum() {
        let quorum_size = 19;
        let mut accumulator = Accumulator::with_capacity(quorum_size, 100);
        let key = random::<i32>();
        let mut noise_keys: Vec<i32> = Vec::with_capacity(5);
        while noise_keys.len() < 5 {
            let noise_key = random::<i32>();
            if noise_key != key {
                noise_keys.push(noise_key);
            };
        }
        for _ in 0..quorum_size - 1 {
            for noise_key in &noise_keys {
                let _ = accumulator.add(*noise_key, random::<u32>());
            }
            assert!(accumulator.add(key, random::<u32>()).is_none());
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, random::<u32>()).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
    }

    #[test]
    fn delete() {
        let mut accumulator = Accumulator::with_capacity(2, 100);

        assert!(accumulator.add(1, 1).is_none());
        assert_eq!(accumulator.contains_key(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), false);

        let mut responses = unwrap_option!(accumulator.get(&1), "");

        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], 1);

        accumulator.delete(&1);

        assert!(accumulator.get(&1).is_none());

        assert!(accumulator.add(1, 1).is_none());
        assert_eq!(accumulator.contains_key(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), false);
        assert!(accumulator.add(1, 1).is_some());
        assert_eq!(accumulator.contains_key(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), true);

        responses = unwrap_option!(accumulator.get(&1), "");

        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0], 1);
        assert_eq!(responses[1], 1);

        accumulator.delete(&1);

        assert!(accumulator.get(&1).is_none());
    }

    #[test]
    fn fill() {
        let mut accumulator = Accumulator::with_capacity(1, 1000);

        for count in 0..1000 {
            assert!(accumulator.add(count, 1).is_some());
            assert_eq!(accumulator.contains_key(&count), true);
            assert_eq!(accumulator.is_quorum_reached(&count), true);
        }

        for count in 0..1000 {
            let responses = unwrap_option!(accumulator.get(&count), "");
            assert_eq!(responses.len(), 1);
            assert_eq!(responses[0], 1);
        }
    }

    #[test]
    fn cache_removals() {
        let mut accumulator = Accumulator::with_capacity(2, 1000);

        for count in 0..1000 {
            assert!(accumulator.add(count, 1).is_none());
            assert_eq!(accumulator.contains_key(&count), true);
            assert_eq!(accumulator.is_quorum_reached(&count), false);

            let responses = unwrap_option!(accumulator.get(&count), "");

            assert_eq!(responses.len(), 1);
            assert_eq!(responses[0], 1);
        }

        assert!(accumulator.add(1000, 1).is_none());
        assert_eq!(accumulator.contains_key(&1000), true);
        assert_eq!(accumulator.is_quorum_reached(&1000), false);
        assert_eq!(accumulator.cache_size(), 1000);

        for count in 0..1000 {
            let option = accumulator.get(&count);

            assert!(option.is_none());

            assert!(accumulator.add(count + 1001, 1).is_none());
            assert_eq!(accumulator.contains_key(&(count + 1001)), true);
            assert_eq!(accumulator.is_quorum_reached(&(count + 1001)), false);
            assert_eq!(accumulator.cache_size(), 1000);
        }
    }

    #[test]
    fn set_quorum_size() {
        let mut accumulator: Accumulator<i32, u32> = Accumulator::with_capacity(2, 100);
        let random = random::<usize>();
        accumulator.set_quorum_size(random);
        assert_eq!(random, accumulator.quorum);
    }
}
