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
#![doc(html_logo_url = "http://maidsafe.net/img/Resources/branding/maidsafe_logo.fab2.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
              html_root_url = "http://dirvine.github.io/accumulator")]
#![feature(negate_unsigned)]
#![forbid(bad_style, missing_docs, warnings)]
#![deny(deprecated, improper_ctypes, non_shorthand_field_patterns,
        overflowing_literals, plugin_as_library, private_no_mangle_fns, private_no_mangle_statics,
        raw_pointer_derive, stable_features, unconditional_recursion, unknown_lints,
        unsafe_code, unused, unused_allocation, unused_attributes,
        unused_comparisons, unused_features, unused_parens, while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, variant_size_differences)]

//! An accumulator container based on an Lru cach (time and size controlled)
//! This container accumulate keys until a number of entries is reached.
//!
//!

extern crate lru_time_cache;
use lru_time_cache::LruCache;

/// entry in the accumulator
#[derive(Clone)]
pub struct Entry<V> {
    /// Expected threshold for resolve
    pub received_response: Vec<V>,
}

/// Accumulator for various message types
pub struct Accumulator<K, V> where K: PartialOrd + Ord + Clone, V: Clone {
    /// Expected threshold for resolve
    quorum: usize,
    storage: LruCache<K, Entry<V>>
}

impl<K: PartialOrd + Ord + Clone, V: Clone> Accumulator<K, V> {
    /// Construct an accumulator and pass size to accumulate unil
    pub fn new(quorum: usize) -> Accumulator<K, V> {
        Accumulator { quorum: quorum, storage: LruCache::<K, Entry<V>>::with_capacity(1000) }
    }
    /// Check for existence of any key and `refresh` the key in the LRU cache.
    // TODO: I think this one should be deprecated in favor of the `contains_key` function.
    pub fn have_name(&mut self, name: &K) -> bool {
        self.storage.get(name).is_some()
    }

    /// Check for existence of any key
    pub fn contains_key(&self, name: &K) -> bool {
        self.storage.check(name)
    }

    /// Check if requested size is accumulated
    pub fn is_quorum_reached(&mut self, name: &K) -> bool {
        let entry = self.storage.get(name);

        if entry.is_none() {
            false
        } else {
            entry.unwrap().received_response.len() >= self.quorum
        }
    }
    /// Add a key / value pair, returns key and vector of values if size reached
    pub fn add(&mut self, name: K, value: V)-> Option<(K, Vec<V>)> {
        let entry = self.storage.remove(&name);
        if entry.is_none() {
            let entry_in = Entry { received_response : vec![value]};
            self.storage.add(name.clone(), entry_in.clone());
            if self.quorum == 1 {
                let result = (name, entry_in.received_response);
                return Some(result);
            }
        } else {
            let mut tmp = entry.unwrap();
            tmp.received_response.push(value);
            self.storage.add(name.clone(), tmp.clone());
            if tmp.received_response.len() >= self.quorum {
                return Some((name, tmp.received_response));
            }
        }
        None
    }
    /// Retrieve a ky/value from the store
    pub fn get(&mut self, name: &K) -> Option<(K, Vec<V>)>{
        let entry = self.storage.get(name);
        if entry.is_none() {
            None
        } else {
            Some((name.clone(), entry.unwrap().received_response.clone()))
        }
    }
    /// Remove an entry (all values for a key will be removed)
    pub fn delete(&mut self, name: &K) {
        self.storage.remove(name);
    }
    /// Return size of container
    pub fn cache_size(&mut self) -> usize {
        self.storage.len()
    }
    /// Sets new size for quorum
    pub fn set_quorum_size(&mut self, new_size: usize) {
        self.quorum = new_size;
    }
}

#[cfg(test)]
mod test {
    extern crate rand;
    use super::*;

    #[test]
    fn add() {
        let mut accumulator : Accumulator<i32, u32> = Accumulator::new(1);

        assert!(accumulator.add(2, 3).is_some());
        assert_eq!(accumulator.have_name(&1), false);
        assert_eq!(accumulator.have_name(&2), true);
        assert_eq!(accumulator.is_quorum_reached(&1), false);
        assert_eq!(accumulator.is_quorum_reached(&2), true);
        assert!(accumulator.add(1, 3).is_some());
        assert_eq!(accumulator.have_name(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), true);
        assert!(accumulator.add(1, 3).is_some());
        assert_eq!(accumulator.have_name(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), true);

        let (key, responses) = accumulator.get(&1).unwrap();

        assert_eq!(key, 1);
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0], 3);
        assert_eq!(responses[1], 3);

        let (key, responses) = accumulator.get(&2).unwrap();

        assert_eq!(key, 2);
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], 3);
    }

    #[test]
    fn add_single_value_quorum() {
        let quorum_size : usize = 19;
        let mut accumulator : Accumulator<i32, u32> = Accumulator::new(quorum_size);
        let key = rand::random::<i32>();
        let value = rand::random::<u32>();
        for i in 0..quorum_size-1 {
            assert!(accumulator.add(key, value).is_none());
            let key_value = accumulator.get(&key).unwrap();
            assert_eq!(key_value.0, key);
            assert_eq!(key_value.1.len(), i + 1);
            for response in key_value.1 { assert_eq!(response, value); };
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, value).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
        let key_value = accumulator.get(&key).unwrap();
        assert_eq!(key_value.0, key);
        assert_eq!(key_value.1.len(), quorum_size);
        for response in key_value.1 { assert_eq!(response, value); };
    }

    #[test]
    fn add_multiple_values_quorum() {
        let quorum_size : usize = 19;
        let mut accumulator : Accumulator<i32, u32> = Accumulator::new(quorum_size);
        let key = rand::random::<i32>();
        for _ in 0..quorum_size-1 {
            assert!(accumulator.add(key, rand::random::<u32>()).is_none());
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, rand::random::<u32>()).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
    }

    #[test]
    fn add_multiple_keys_quorum() {
        let quorum_size : usize = 19;
        let mut accumulator : Accumulator<i32, u32> = Accumulator::new(quorum_size);
        let key = rand::random::<i32>();
        let mut noise_keys : Vec<i32> = Vec::with_capacity(5);
        while noise_keys.len() < 5 {
            let noise_key = rand::random::<i32>();
            if noise_key != key { noise_keys.push(noise_key); }; };
        for _ in 0..quorum_size-1 {
            for noise_key in noise_keys.iter() {
                accumulator.add(noise_key.clone(), rand::random::<u32>());
            }
            assert!(accumulator.add(key, rand::random::<u32>()).is_none());
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, rand::random::<u32>()).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
    }

    #[test]
    fn delete() {
        let mut accumulator : Accumulator<i32, u32> = Accumulator::new(2);

        assert!(accumulator.add(1, 1).is_none());
        assert_eq!(accumulator.have_name(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), false);

        let (key, responses) = accumulator.get(&1).unwrap();

        assert_eq!(key, 1);
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], 1);

        accumulator.delete(&1);

        let option = accumulator.get(&1);

        assert!(option.is_none());

        assert!(accumulator.add(1, 1).is_none());
        assert_eq!(accumulator.have_name(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), false);
        assert!(accumulator.add(1, 1).is_some());
        assert_eq!(accumulator.have_name(&1), true);
        assert_eq!(accumulator.is_quorum_reached(&1), true);

        let (key, responses) = accumulator.get(&1).unwrap();

        assert_eq!(key, 1);
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0], 1);
        assert_eq!(responses[1], 1);

        accumulator.delete(&1);

        let option = accumulator.get(&1);

        assert!(option.is_none());
    }

    #[test]
    fn fill() {
        let mut accumulator : Accumulator<i32, u32> = Accumulator::new(1);

        for count in 0..1000 {
            assert!(accumulator.add(count, 1).is_some());
            assert_eq!(accumulator.have_name(&count), true);
            assert_eq!(accumulator.is_quorum_reached(&count), true);
        }

        for count in 0..1000 {
            let (key, responses) = accumulator.get(&count).unwrap();

            assert_eq!(key, count);
            assert_eq!(responses.len(), 1);
            assert_eq!(responses[0], 1);
        }
    }

    #[test]
    fn cache_removals() {
        let mut accumulator : Accumulator<i32, u32> = Accumulator::new(2);

        for count in 0..1000 {
            assert!(accumulator.add(count, 1).is_none());
            assert_eq!(accumulator.have_name(&count), true);
            assert_eq!(accumulator.is_quorum_reached(&count), false);

            let (key, responses) = accumulator.get(&count).unwrap();

            assert_eq!(key, count);
            assert_eq!(responses.len(), 1);
            assert_eq!(responses[0], 1);
            assert_eq!(accumulator.cache_size(), count as usize + 1);
        }

        assert!(accumulator.add(1000, 1).is_none());
        assert_eq!(accumulator.have_name(&1000), true);
        assert_eq!(accumulator.is_quorum_reached(&1000), false);
        assert_eq!(accumulator.cache_size(), 1000);

        for count in 0..1000 {
            let option = accumulator.get(&count);

            assert!(option.is_none());

            assert!(accumulator.add(count + 1001, 1).is_none());
            assert_eq!(accumulator.have_name(&(count + 1001)), true);
            assert_eq!(accumulator.is_quorum_reached(&(count + 1001)), false);
            assert_eq!(accumulator.cache_size(), 1000);
        }
    }

    #[test]
    fn set_quorum_size() {
        let mut accumulator : Accumulator<i32, u32> = Accumulator::new(2);
        let random = rand::random::<usize>();
        accumulator.set_quorum_size(random);
        assert_eq!(random, accumulator.quorum);
    }
}
