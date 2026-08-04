use alloc::vec::Vec;

pub type SmallSetIter<'a, T> = core::slice::Iter<'a, T>;

/// Linear-scan set backed by a `Vec<T>`. Cheaper for sets that only store a handful of entries
/// than a [`BTreeSet`].
#[derive(Debug, Clone)]
pub struct SmallSet<T>(Vec<T>);

impl<T> Default for SmallSet<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: PartialEq> SmallSet<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.0.contains(value)
    }

    /// Returns `true` if the value wasn't already present.
    pub fn insert(&mut self, value: T) -> bool {
        if self.0.contains(&value) {
            false
        } else {
            self.0.push(value);
            true
        }
    }

    /// Returns `true` if the value was present.
    pub fn remove(&mut self, value: &T) -> bool {
        match self.0.iter().position(|v| v == value) {
            Some(index) => {
                self.0.swap_remove(index);
                true
            }
            None => false,
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.0.iter()
    }
}

impl<T> IntoIterator for SmallSet<T> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::SmallSet;

    #[test]
    fn contains_on_empty_set_is_false() {
        let set: SmallSet<u32> = SmallSet::new();
        assert!(!set.contains(&1));
    }

    #[test]
    fn insert_new_value_returns_true_and_is_visible() {
        let mut set = SmallSet::new();
        assert!(set.insert(1));
        assert!(set.contains(&1));
    }

    #[test]
    fn insert_duplicate_value_returns_false_and_does_not_grow() {
        let mut set = SmallSet::new();
        set.insert(1);
        assert!(!set.insert(1));
        assert_eq!(set.iter().count(), 1);
    }

    #[test]
    fn remove_returns_true_once_then_false() {
        let mut set = SmallSet::new();
        set.insert(1);
        assert!(set.remove(&1));
        assert!(!set.contains(&1));
        assert!(!set.remove(&1));
    }

    #[test]
    fn remove_preserves_other_elements() {
        let mut set = SmallSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);
        set.remove(&2);
        assert!(set.contains(&1));
        assert!(!set.contains(&2));
        assert!(set.contains(&3));
    }

    #[test]
    fn iter_sees_every_element() {
        let mut set = SmallSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);
        let mut elements: Vec<_> = set.iter().copied().collect();
        elements.sort();
        assert_eq!(elements, [1, 2, 3]);
    }

    #[test]
    fn into_iter_yields_owned_elements() {
        let mut set = SmallSet::new();
        set.insert("a");
        set.insert("b");
        let mut elements: Vec<_> = set.into_iter().collect();
        elements.sort();
        assert_eq!(elements, ["a", "b"]);
    }
}
