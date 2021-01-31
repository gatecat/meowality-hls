use std::hash::Hash;
use rustc_hash::FxHashMap;

/*
A set with integer indices given to items based on insertion order
used e.g. for IdStrings
*/

pub struct IndexedSet<T: Eq + Hash + Clone> {
	keys: Vec<T>,
	key_to_index: FxHashMap<T, usize>,
}

impl<T: Eq + Hash + Clone> IndexedSet<T> {
	pub fn new() -> IndexedSet<T> {
		IndexedSet::<T> {
			keys: Vec::new(),
			key_to_index: FxHashMap::default(),
		}
	}

	pub fn add(&mut self, key: &T) -> usize {
		// Add an item (if it doesn't already exist) and return its index
		match self.key_to_index.get(key) {
			Some(k) => *k,
			None => {
				let index = self.keys.len();
				self.keys.push(key.clone());
				self.key_to_index.insert(key.clone(), index);
				return index;
			}
		}
	}

	pub fn get_index(&self, key: &T) -> Option<usize> {
		self.key_to_index.get(key).cloned()
	}

	pub fn key(&self, index: usize) -> &T {
		&self.keys[index]
	}
	pub fn iter(&self) -> std::slice::Iter<T> {
		self.keys.iter()
	}
	pub fn len(&self) -> usize {
		self.keys.len()
	}
}


#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn basics() {
		let mut set :IndexedSet<String> = IndexedSet::new();
		assert_eq!(set.add(&"foo".to_string()), 0);
		assert_eq!(set.add(&"bar".to_string()), 1);
		assert_eq!(set.add(&"foo".to_string()), 0);
		assert_eq!(set.len(), 2);
		assert_eq!(set.key(1), "bar");
		assert_eq!(set.get_index(&"foo".to_string()), Some(0));
	}
	#[test]
	fn iterator() {
		let mut set :IndexedSet<String> = IndexedSet::new();
		assert_eq!(set.add(&"foo".to_string()), 0);
		assert_eq!(set.add(&"bar".to_string()), 1);
		assert_eq!(set.iter().nth(0).unwrap(), "foo");
		assert_eq!(set.iter().nth(1).unwrap(), "bar");
	}
}