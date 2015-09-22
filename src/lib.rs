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

#![forbid(
    bad_style,              // Includes:
                            // - non_camel_case_types:   types, variants, traits and type parameters
                            //                           should have camel case names,
                            // - non_snake_case:         methods, functions, lifetime parameters and
                            //                           modules should have snake case names
                            // - non_upper_case_globals: static constants should have uppercase
                            //                           identifiers
    exceeding_bitshifts,    // shift exceeds the type's number of bits
    mutable_transmutes,     // mutating transmuted &mut T from &T may cause undefined behavior
    no_mangle_const_items,  // const items will not have their symbols exported
    unknown_crate_types,    // unknown crate type found in #[crate_type] directive
    warnings                // mass-change the level for lints which produce warnings
    )]

#![deny(
    deprecated,                    // detects use of #[deprecated] items
    drop_with_repr_extern,         // use of #[repr(C)] on a type that implements Drop
    improper_ctypes,               // proper use of libc types in foreign modules
    missing_docs,                  // detects missing documentation for public members
    non_shorthand_field_patterns,  // using `Struct { x: x }` instead of `Struct { x }`
    overflowing_literals,          // literal out of range for its type
    plugin_as_library,             // compiler plugin used as ordinary library in non-plugin crate
    private_no_mangle_fns,         // functions marked #[no_mangle] should be exported
    private_no_mangle_statics,     // statics marked #[no_mangle] should be exported
    raw_pointer_derive,            // uses of #[derive] with raw pointers are rarely correct
    stable_features,               // stable features found in #[feature] directive
    unconditional_recursion,       // functions that cannot return without calling themselves
    unknown_lints,                 // unrecognized lint attribute
    unsafe_code,                   // usage of `unsafe` code
    unused,                        // Includes:
                                   // - unused_imports:     imports that are never used
                                   // - unused_variables:   detect variables which are not used in
                                   //                       any way
                                   // - unused_assignments: detect assignments that will never be
                                   //                       read
                                   // - dead_code:          detect unused, unexported items
                                   // - unused_mut:         detect mut variables which don't need to
                                   //                       be mutable
                                   // - unreachable_code:   detects unreachable code paths
                                   // - unused_must_use:    unused result of a type flagged as
                                   //                       #[must_use]
                                   // - unused_unsafe:      unnecessary use of an `unsafe` block
                                   // - path_statements: path statements with no effect
    unused_allocation,             // detects unnecessary allocations that can be eliminated
    unused_attributes,             // detects attributes that were not used by the compiler
    unused_comparisons,            // comparisons made useless by limits of the types involved
    unused_features,               // unused or unknown features found in crate-level #[feature]
                                   // directives
    unused_parens,                 // `if`, `match`, `while` and `return` do not need parentheses
    while_true                     // suggest using `loop { }` instead of `while true { }`
    )]

#![warn(
    trivial_casts,            // detects trivial casts which could be removed
    trivial_numeric_casts,    // detects trivial casts of numeric types which could be removed
    unused_extern_crates,     // extern crates that are never used
    unused_import_braces,     // unnecessary braces around an imported item
    unused_qualifications,    // detects unnecessarily qualified names
    unused_results,           // unused result of an expression in a statement
    variant_size_differences  // detects enums with widely varying variant sizes
    )]

#![allow(
    box_pointers,                  // use of owned (Box type) heap memory
    fat_ptr_transmutes,            // detects transmutes of fat pointers
    missing_copy_implementations,  // detects potentially-forgotten implementations of `Copy`
    missing_debug_implementations  // detects missing implementations of fmt::Debug
    )]

// Non-MaidSafe crates
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate rand;
extern crate time;

// MaidSafe crates
extern crate lru_time_cache;

/// Implementation of [Accumulator](index.html#accumulator).
pub struct Accumulator<Key, Value>
    where Key: PartialOrd + Ord + Clone,
          Value: Clone
{
    // Expected threshold for resolve
    quorum: usize,
    lru_cache: ::lru_time_cache::LruCache<Key, Vec<Value>>,
}

impl<Key: PartialOrd + Ord + Clone, Value: Clone> Accumulator<Key, Value> {
    /// Constructor for capacity based `Accumulator`.
    ///
    /// `quorum` defines the count at and above which [`add()`](#method.add) will return `Some()`.
    pub fn with_capacity(quorum: usize, capacity: usize) -> Accumulator<Key, Value> {
        Accumulator {
            quorum: quorum,
            lru_cache: ::lru_time_cache::LruCache::<Key, Vec<Value>>::with_capacity(capacity),
        }
    }

    /// Constructor for time based `Accumulator`.
    ///
    /// `quorum` defines the count at and above which [`add()`](#method.add) will return `Some()`.
    pub fn with_duration(quorum: usize, duration: ::time::Duration) -> Accumulator<Key, Value> {
        Accumulator {
            quorum: quorum,
            lru_cache:
                ::lru_time_cache::LruCache::<Key, Vec<Value>>::with_expiry_duration(duration),
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
            match self.lru_cache.get_mut(&key) {
                Some(result) => result.push(value),
                None => debug!("key found cannot push to value"),
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
    #[test]
    fn add() {
        let mut accumulator = super::Accumulator::with_capacity(1, 100);

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

        let responses = accumulator.get(&1).unwrap();

        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0], 3);

        let responses = accumulator.get(&2).unwrap();

        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], 3);
    }

    #[test]
    fn add_single_value_quorum() {
        let quorum_size = 19;
        let mut accumulator = super::Accumulator::with_capacity(quorum_size, 100);
        let key = ::rand::random::<i32>();
        let value = ::rand::random::<u32>();
        for i in 0..quorum_size - 1 {
            assert!(accumulator.add(key, value).is_none());
            let value = accumulator.get(&key).unwrap();
            assert_eq!(value.len(), i + 1);
            // for response in value { assert_eq!(response, value); };
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, value).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
        let value = accumulator.get(&key).unwrap();
        assert_eq!(value.len(), quorum_size);
        // for response in value { assert_eq!(response, value); };
    }

    #[test]
    fn add_multiple_values_quorum() {
        let quorum_size = 19;
        let mut accumulator = super::Accumulator::with_capacity(quorum_size, 100);
        let key = ::rand::random::<i32>();
        for _ in 0..quorum_size - 1 {
            assert!(accumulator.add(key, ::rand::random::<u32>()).is_none());
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, ::rand::random::<u32>()).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
    }

    #[test]
    fn add_multiple_keys_quorum() {
        let quorum_size = 19;
        let mut accumulator = super::Accumulator::with_capacity(quorum_size, 100);
        let key = ::rand::random::<i32>();
        let mut noise_keys: Vec<i32> = Vec::with_capacity(5);
        while noise_keys.len() < 5 {
            let noise_key = ::rand::random::<i32>();
            if noise_key != key {
                noise_keys.push(noise_key);
            };
        };
        for _ in 0..quorum_size - 1 {
            for noise_key in noise_keys.iter() {
                let _ = accumulator.add(noise_key.clone(), ::rand::random::<u32>());
            }
            assert!(accumulator.add(key.clone(), ::rand::random::<u32>()).is_none());
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key.clone(), ::rand::random::<u32>()).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
    }

    #[test]
    fn delete() {
        let mut accumulator = super::Accumulator::with_capacity(2, 100);

        assert!(accumulator.add(1, 1).is_none());
        assert_eq!(accumulator.contains_key(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), false);

        let responses = accumulator.get(&1).unwrap();

        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], 1);

        accumulator.delete(&1);

        let option = accumulator.get(&1);

        assert!(option.is_none());

        assert!(accumulator.add(1, 1).is_none());
        assert_eq!(accumulator.contains_key(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), false);
        assert!(accumulator.add(1, 1).is_some());
        assert_eq!(accumulator.contains_key(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), true);

        let responses = accumulator.get(&1).unwrap();

        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0], 1);
        assert_eq!(responses[1], 1);

        accumulator.delete(&1);

        let option = accumulator.get(&1);

        assert!(option.is_none());
    }

    #[test]
    fn fill() {
        let mut accumulator = super::Accumulator::with_capacity(1, 1000);

        for count in 0..1000 {
            assert!(accumulator.add(count, 1).is_some());
            assert_eq!(accumulator.contains_key(&count), true);
            assert_eq!(accumulator.is_quorum_reached(&count), true);
        }

        for count in 0..1000 {
            let responses = accumulator.get(&count).unwrap();
            assert_eq!(responses.len(), 1);
            assert_eq!(responses[0], 1);
        }
    }

    #[test]
    fn cache_removals() {
        let mut accumulator = super::Accumulator::with_capacity(2, 1000);

        for count in 0..1000 {
            assert!(accumulator.add(count, 1).is_none());
            assert_eq!(accumulator.contains_key(&count), true);
            assert_eq!(accumulator.is_quorum_reached(&count), false);

            let responses = accumulator.get(&count).unwrap();

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
        let mut accumulator: super::Accumulator<i32, u32> = super::Accumulator::with_capacity(2,
                                                                                              100);
        let random = ::rand::random::<usize>();
        accumulator.set_quorum_size(random);
        assert_eq!(random, accumulator.quorum);
    }
}
