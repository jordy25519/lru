// use std::collections::HashMap;
use fnv::FnvHashMap as HashMap;
use std::{hash::Hash, collections::VecDeque};

trait LRUCache<K, V> {
    fn initialize(max: u16) -> Self;
    fn get(&mut self, k: K) -> Option<V>;
    fn set(&mut self, k: K, v: V);
}

pub struct Cache<K: Eq + Hash, V> {
    /// generation index
    g: u32,
    /// max capacity of the cache
    max: u16,
    /// cached items
    items: HashMap<K, (V, u32)>,
    recency_buckets: VecDeque<(u32, K)>,
}

impl<'a, K: Eq + Hash + Clone, V: Copy> LRUCache<K, V> for Cache<K, V> {
    fn initialize(max: u16) -> Self {
        Self {
            g: 0,
            max,
            items: HashMap::<K, (V, u32)>::with_capacity_and_hasher(
                max as usize,
                Default::default(),
            ),
            recency_buckets: VecDeque::with_capacity(max as usize),
        }
    }
    fn get(&mut self, k: K) -> Option<V> {
        let entry = self.items.get_mut(&k);
        if let Some(entry) = entry {
            let recency = self.g + 1;

            // update recency info for the key
            // keeping it sorted allows O(1) expiry later
            if let Ok(idx) = self.recency_buckets.binary_search_by(|(g, _k)| g.cmp(&entry.1)) {
                if idx == 0 {
                    let _ = self.recency_buckets.pop_front();
                } else {
                    // worst case is (O(N / 2)) since it shift on the shorter side
                    self.recency_buckets.remove(idx);
                }
            }
            self.recency_buckets.push_back((recency, k));

            entry.1 = recency;
            self.g = recency;

            Some(entry.0)
        } else {
            None
        }
    }
    fn set(&mut self, k: K, v: V) {
        if self.items.contains_key(&k) {
            return
        }

        // evict
        if self.items.len() + 1 > self.max as usize {
            let evict = self.recency_buckets.pop_front().unwrap();
            self.items.remove(&evict.1);
        }

        self.recency_buckets.push_back((self.g, k.clone()));
        self.items.insert(k, (v, self.g));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut cache = Cache::initialize(3);
        cache.set(1, "a");
        cache.set(2, "b");
        cache.set(3, "c");
        assert_eq!(cache.get(1).unwrap(), "a");
        assert_eq!(cache.get(2).unwrap(), "b");
    }

    #[test]
    fn expires_lru() {
        let mut cache = Cache::initialize(3);
        cache.set(1, "a");
        let _ = cache.get(1);
        println!("{:?}", cache.items);
        println!("{:?}", cache.recency_buckets);
        cache.set(2, "b");
        let _ = cache.get(2);
        println!("{:?}", cache.items);
        println!("{:?}", cache.recency_buckets);
        cache.set(3, "c");
        println!("{:?}", cache.items);
        println!("{:?}", cache.recency_buckets);
        let _ = cache.get(1);
        println!("{:?}", cache.items);
        println!("{:?}", cache.recency_buckets);
        let _ = cache.get(2);
        println!("{:?}", cache.items);
        println!("{:?}", cache.recency_buckets);
        cache.set(4, "d");
        println!("{:?}", cache.items);
        println!("{:?}", cache.recency_buckets);
        assert_eq!(cache.items.len(), cache.max as usize);
        assert!(cache.get(3).is_none());
    }

    // #[test]
    // fn expires_lru_bench() {
    //     let count = 100_000;
    //     let mut cache = Cache::initialize(count);

    //     for c in 0..count {
    //         cache.set(1, "a");
    //         cache.set(2, "b");
    //         cache.set(3, "c");
    //         cache.set(4, "d");
    //     }
    // }
}

// hashmap cache, recency update on get O(1), set: hashmap iterate all keys to expire oldest O(N)

// hashmap cache + recency queue, recency update on get O(log N), set: lookup to expire O(1)
    // figure out best swap_remove so we don't cause left shift of vec

// hashmap cache + splay tree, recency update on get O(log N), set: lookup to expire O(1)