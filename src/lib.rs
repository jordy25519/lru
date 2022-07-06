// use std::collections::HashMap;
use fnv::FnvHashMap as HashMap;
use std::hash::Hash;

trait LRUCache<K, V> {
    fn initialize(max: u16) -> Self;
    fn get(&mut self, k: K) -> Option<V>;
    fn set(&mut self, k: K, v: V);
}

pub struct Cache<K: Eq + Hash, V> {
    g: u32,
    max: u16,
    items: HashMap<K, (V, u32)>,
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
        }
    }
    fn get(&mut self, k: K) -> Option<V> {
        let entry = self.items.get_mut(&k);
        if let Some(entry) = entry {
            entry.1 = self.g + 1;
            self.g += 1;
            Some(entry.0)
        } else {
            None
        }
    }
    fn set(&mut self, k: K, v: V) {
        let mut oldest_g = u32::max_value();
        let mut evict: Option<K> = None;
        if self.items.len() + 1 > self.max as usize {
            for (k, v) in self.items.iter() {
                if v.1 < oldest_g {
                    evict = Some(k.clone());
                    oldest_g = v.1;
                }
            }
        }
        if let Some(ref evict) = evict {
            self.items.remove(evict);
        }

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
        cache.set(2, "b");
        cache.set(3, "c");
        let _ = cache.get(1);
        let _ = cache.get(2);
        cache.set(4, "d");
        assert_eq!(cache.items.len(), cache.max as usize);
        assert!(cache.get(3).is_none());
    }
}
