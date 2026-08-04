use alloc::vec::Vec;

/// Linear-scan map backed by a `Vec<(K, V)>`. Cheaper than a `BTreeMap`
/// for collections that never hold more than a handful of entries.
#[derive(Debug, Clone)]
pub struct SmallMap<K, V>(Vec<(K, V)>);

impl<K, V> Default for SmallMap<K, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<K: PartialEq, V> SmallMap<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.0.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => Some(core::mem::replace(v, value)),
            None => {
                self.0.push((key, value));
                None
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.0.iter().position(|(k, _)| k == key)?;
        Some(self.0.swap_remove(index).1)
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        Entry { map: self, key }
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }

    pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, mut f: F) {
        self.0.retain_mut(|(k, v)| f(k, v));
    }
}

impl<K, V> IntoIterator for SmallMap<K, V> {
    type Item = (K, V);
    type IntoIter = alloc::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K: PartialEq, V> FromIterator<(K, V)> for SmallMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = Self::default();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

pub struct Entry<'a, K, V> {
    map: &'a mut SmallMap<K, V>,
    key: K,
}

impl<'a, K: PartialEq, V> Entry<'a, K, V> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }

    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }

    pub fn or_insert_with(self, f: impl FnOnce() -> V) -> &'a mut V {
        let index = match self.map.0.iter().position(|(k, _)| *k == self.key) {
            Some(index) => index,
            None => {
                self.map.0.push((self.key, f()));
                self.map.0.len() - 1
            }
        };
        &mut self.map.0[index].1
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::SmallMap;

    #[test]
    fn get_on_empty_map_is_none() {
        let map: SmallMap<u32, &str> = SmallMap::new();
        assert_eq!(map.get(&1), None);
    }

    #[test]
    fn insert_then_get_roundtrips() {
        let mut map = SmallMap::new();
        assert_eq!(map.insert(1, "a"), None);
        assert_eq!(map.get(&1), Some(&"a"));
    }

    #[test]
    fn insert_existing_key_replaces_and_returns_old_value() {
        let mut map = SmallMap::new();
        map.insert(1, "a");
        assert_eq!(map.insert(1, "b"), Some("a"));
        assert_eq!(map.get(&1), Some(&"b"));
    }

    #[test]
    fn remove_returns_value_and_forgets_key() {
        let mut map = SmallMap::new();
        map.insert(1, "a");
        assert_eq!(map.remove(&1), Some("a"));
        assert_eq!(map.get(&1), None);
        assert_eq!(map.remove(&1), None);
    }

    #[test]
    fn remove_preserves_other_entries() {
        let mut map = SmallMap::new();
        map.insert(1, "a");
        map.insert(2, "b");
        map.insert(3, "c");
        map.remove(&2);
        assert_eq!(map.get(&1), Some(&"a"));
        assert_eq!(map.get(&2), None);
        assert_eq!(map.get(&3), Some(&"c"));
    }

    #[test]
    fn entry_or_default_inserts_on_miss_and_reuses_on_hit() {
        let mut map: SmallMap<u32, u32> = SmallMap::new();
        *map.entry(1).or_default() += 1;
        *map.entry(1).or_default() += 1;
        assert_eq!(map.get(&1), Some(&2));
    }

    #[test]
    fn entry_or_insert_with_only_calls_closure_on_miss() {
        let mut map: SmallMap<u32, u32> = SmallMap::new();
        let mut calls = 0;
        map.entry(1).or_insert_with(|| {
            calls += 1;
            10
        });
        map.entry(1).or_insert_with(|| {
            calls += 1;
            20
        });
        assert_eq!(map.get(&1), Some(&10));
        assert_eq!(calls, 1);
    }

    #[test]
    fn keys_values_iter_see_every_entry() {
        let map: SmallMap<u32, &str> = [(1, "a"), (2, "b")].into_iter().collect();
        let mut keys: Vec<_> = map.keys().copied().collect();
        keys.sort();
        assert_eq!(keys, vec![1, 2]);

        let mut values: Vec<_> = map.values().copied().collect();
        values.sort();
        assert_eq!(values, vec!["a", "b"]);

        assert_eq!(map.iter().count(), 2);
    }

    #[test]
    fn retain_keeps_only_matching_entries() {
        let mut map: SmallMap<u32, u32> = (0..5).map(|n| (n, n * 10)).collect();
        map.retain(|k, _| k % 2 == 0);
        let mut remaining: Vec<_> = map.keys().copied().collect();
        remaining.sort();
        assert_eq!(remaining, vec![0, 2, 4]);
    }

    #[test]
    fn from_iter_with_duplicate_keys_keeps_last_value() {
        let map: SmallMap<u32, &str> = [(1, "a"), (1, "b")].into_iter().collect();
        assert_eq!(map.get(&1), Some(&"b"));
        assert_eq!(map.iter().count(), 1);
    }

    #[test]
    fn into_iter_yields_owned_pairs() {
        let map: SmallMap<u32, &str> = [(1, "a"), (2, "b")].into_iter().collect();
        let mut pairs: Vec<_> = map.into_iter().collect();
        pairs.sort();
        assert_eq!(pairs, vec![(1, "a"), (2, "b")]);
    }
}
