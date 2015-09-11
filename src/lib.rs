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
#![doc(html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
              html_root_url = "http://maidsafe.github.io/accumulator")]
#![forbid(bad_style, missing_docs, warnings)]
#![deny(deprecated, improper_ctypes, non_shorthand_field_patterns,
        overflowing_literals, plugin_as_library, private_no_mangle_fns, private_no_mangle_statics,
        raw_pointer_derive, stable_features, unconditional_recursion, unknown_lints,
        unsafe_code, unused, unused_allocation, unused_attributes,
        unused_comparisons, unused_features, unused_parens, while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, variant_size_differences)]

//! An accumulator container based on an Lru cach (time and size controlled)
//! This container accumulate keys *until* a number of entries is reached.
//! After this quaorum has been reached the container will continue to accept values for such keys
//! this allows users to test merge functions until they are happy they have a good value.
//! Otherwise a hacker could pass a single bad value and break all quorums

extern crate lru_time_cache;
extern crate time;
#[macro_use] extern crate log;
use lru_time_cache::LruCache;
use time::Duration;

/// Accumulator for various message types
pub struct Accumulator<K, V> where K: PartialOrd + Ord + Clone, V: Clone {
    /// Expected threshold for resolve
    quorum: usize,
    lru_cache: LruCache<K, Vec<V>>
}

impl<K: PartialOrd + Ord + Clone, V: Clone> Accumulator<K, V> {
    /// Construct an accumulator and pass size to accumulate unil
    pub fn with_capacity(quorum: usize, capacity: usize) -> Accumulator<K, V> {
        Accumulator { quorum: quorum, lru_cache: LruCache::<K, Vec<V>>::with_capacity(capacity) }
    }

    /// Construct an accumulator and pass duration to accumulate until
    pub fn with_duration(quorum: usize, duration: Duration) -> Accumulator<K, V> {
        Accumulator { quorum: quorum, lru_cache: LruCache::<K, Vec<V>>::with_expiry_duration(duration) }
    }

    /// Check for existence of any key
    pub fn contains_key(&self, name: &K) -> bool {
        self.lru_cache.contains_key(name)
    }

    /// Check if requested size is accumulated
    pub fn is_quorum_reached(&mut self, name: &K) -> bool {
        match self.lru_cache.get(name) {
        None => false,
        Some(entry) => entry.len() >= self.quorum    
        }
    }
    // /// Check if requested size will be accumulated on this attempt
    // fn will_reach_quorum(&mut self, name: &K) -> bool {
    //     match self.lru_cache.get(name) {
    //     None => self.quorum == 1 || false,
    //     Some(entry) => entry.received_response.len() + 1 == self.quorum     
    //     }
    // }
    /// Add a key / value pair, returns key and vector of values if size reached
    /// if already reached then keep adding to this value (we cannot tell values are all valid)
    pub fn add(&mut self, key: K, value: V)-> Option<Vec<V>> {

        if self.contains_key(&key) {
            match self.lru_cache.get_mut(&key) {
                Some(result) => result.push(value),
                    None => debug!("key found cannot push to value")
            }
        } else {
            self.lru_cache.add(key.clone(), vec![value]);
        }

        // FIXME(dirvine) This iterates to many times, should combine and answer in one iteration :27/08/2015
        if self.is_quorum_reached(&key) {
            match self.lru_cache.get(&key) {
                Some(value) => Some(value.clone()),
                    None => None    
            }
        } else {
            None
        }
    }
    /// Retrieve a ky/value from the store
    pub fn get(&mut self, name: &K) -> Option<Vec<V>>{
        match self.lru_cache.get(name) {
            Some(entry) => Some(entry.clone()),
                None => None    
        }
    }
    /// Remove an entry (all values for a key will be removed)
    pub fn delete(&mut self, name: &K) {
        self.lru_cache.remove(name);
    }
    /// Return size of container
    pub fn cache_size(&mut self) -> usize {
        self.lru_cache.len()
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
    use self::rand::random; 

    #[test]
    fn add() {
        let mut accumulator : Accumulator<i32, u32> = Accumulator::with_capacity(1, 100);

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
        let mut accumulator : Accumulator<i32, u32> = Accumulator::with_capacity(quorum_size, 100);
        let key = random::<i32>();
        let value = random::<u32>();
        for i in 0..quorum_size-1 {
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
        let quorum_size  = 19;
        let mut accumulator : Accumulator<i32, u32> = Accumulator::with_capacity(quorum_size, 100);
        let key = random::<i32>();
        for _ in 0..quorum_size -1 {
            assert!(accumulator.add(key, random::<u32>()).is_none());
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key, random::<u32>()).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
    }

    #[test]
    fn add_multiple_keys_quorum() {
        let quorum_size = 19;
        let mut accumulator : Accumulator<i32, u32> = Accumulator::with_capacity(quorum_size, 100);
        let key = random::<i32>();
        let mut noise_keys : Vec<i32> = Vec::with_capacity(5);
        while noise_keys.len() < 5 {
            let noise_key = random::<i32>();
            if noise_key != key { noise_keys.push(noise_key); }; 
        };
        for _ in 0..quorum_size -1 {
            for noise_key in noise_keys.iter() {
                accumulator.add(noise_key.clone(), random::<u32>());
            }
            assert!(accumulator.add(key.clone(), random::<u32>()).is_none());
            assert_eq!(accumulator.is_quorum_reached(&key), false);
        }
        assert!(accumulator.add(key.clone(), random::<u32>()).is_some());
        assert_eq!(accumulator.is_quorum_reached(&key), true);
    }

    #[test]
    fn delete() {
        let mut accumulator : Accumulator<i32, u32> = Accumulator::with_capacity(2, 100);

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
        let mut accumulator : Accumulator<i32, u32> = Accumulator::with_capacity(1, 1000);

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
        let mut accumulator : Accumulator<i32, u32> = Accumulator::with_capacity(2, 1000);

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
        let mut accumulator : Accumulator<i32, u32> = Accumulator::with_capacity(2, 100);
        let random = random::<usize>();
        accumulator.set_quorum_size(random);
        assert_eq!(random, accumulator.quorum);
    }
}
