/*
An indexed object store, used for fast netlist structures

Consists of a slotted vector. All objects have an index that is valid for their lifetime and should be used as a 'handle'.

*/

use crate::core::{Nullable, NullValue};
use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;

/*
This custom index type can be used down-stream to automically create more useful error messages; and prevent accidental use of an index of one object type being used for another
*/

#[derive(Hash)]
pub struct StoreIndex<T: ?Sized> {
	idx: u32,
	phantom: PhantomData<T>,
}

impl<T> StoreIndex<T> {
	#[inline]
	pub fn from_index(idx: usize) -> StoreIndex<T> {
		StoreIndex {
			idx: idx.try_into().unwrap(),
			phantom: PhantomData,
		}
	}
	#[inline]
	pub fn index(&self) -> usize {
		return self.idx as usize;
	}
}

// Not using derive here; to avoid issues around PhantomData creating excessive constraints

impl<T> Clone for StoreIndex<T> {
	fn clone(&self) -> Self {
		StoreIndex {
			idx: self.idx,
			phantom: PhantomData,
		}
	}
}

impl<T> Copy for StoreIndex<T> {}

impl<T> PartialEq for StoreIndex<T> {
	fn eq(&self, other: &Self) -> bool {
		self.idx == other.idx
	}
}
impl<T> Eq for StoreIndex<T> {}

impl<T> PartialOrd for StoreIndex<T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { self.idx.partial_cmp(&other.idx) }
}
impl<T> Ord for StoreIndex<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.idx.cmp(&other.idx) }
}

impl <T> fmt::Debug for StoreIndex<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "StoreIndex<{}>({})", std::any::type_name::<T>(), self.idx)
	}
}

impl <T> NullValue for StoreIndex<T> {
	const NULL: Self = Self {
		idx: 0xFFFFFFFF,
		phantom: PhantomData,
	};
}

pub type NullableIndex<T> = Nullable<StoreIndex<T>>;

pub struct Slot<T> {
	data: Option<T>,
	next_free: u32,
}

pub struct ObjectStore<T> {
	slots: Vec<Slot<T>>,
	next_free: u32,
	active_count: usize,
}

impl<T> ObjectStore<T> {
	// Create a new, empty store
	pub fn new() -> ObjectStore<T> {
		ObjectStore {
			slots: Vec::new(),
			next_free: 0,
			active_count: 0,
		}
	}
	// Add an object; returning its assigned index
	pub fn add(&mut self, obj: T) -> StoreIndex<T> {
		let idx = self.next_free;
		if (self.next_free as usize) == self.slots.len() {
			self.slots.push(Slot {
				data: Some(obj),
				next_free: idx + 1,
			});
			self.next_free += 1;
		} else {
			let mut slot = &mut self.slots[self.next_free as usize];
			slot.data = Some(obj);
			self.next_free = slot.next_free;
		}
		self.active_count += 1;
		StoreIndex {
			idx: idx,
			phantom: PhantomData {},
		}
	}
	// Get an object by index
	pub fn get(&self, idx: StoreIndex<T>) -> &T {
		self.slots[idx.idx as usize].data.as_ref().unwrap()
	}
	pub fn get_mut(&mut self, idx: StoreIndex<T>) -> &mut T {
		self.slots[idx.idx as usize].data.as_mut().unwrap()
	}
	// Remove an object by index
	pub fn remove(&mut self, idx: StoreIndex<T>) {
		let mut slot = &mut self.slots[idx.idx as usize];
		slot.data = None;
		slot.next_free = self.next_free;
		self.next_free = idx.idx;
		self.active_count -= 1;
	}
	// Iterate over non-emtpy slots; returning (index, value)
	pub fn iter(&self) -> Iter<T> {
		Iter {
			base: self.slots.iter(),
			index: 0,
		}
	}
	pub fn iter_mut(&mut self) -> IterMut<T> {
		IterMut {
			base: self.slots.iter_mut(),
			index: 0,
		}
	}
	// Get the count of objects in the store
	pub fn count(&self) -> usize {
		self.active_count
	}
	// Get the total allocated size of the store (i.e. the maximum range of indices)
	pub fn size(&self) -> usize {
		self.slots.len()
	}
}

pub struct Iter<'a, T> {
	base: std::slice::Iter<'a, Slot<T>>,
	index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = (StoreIndex<T>, &'a T);

	fn next(&mut self) -> Option<(StoreIndex<T>, &'a T)> {
		loop {
			let base_next = self.base.next();
			match base_next {
				None => {
					return None;
				}
				Some(slot) => {
					let curr_idx = self.index;
					self.index += 1;
					match &slot.data {
						Some(t) => {
							return Some((StoreIndex::from_index(curr_idx), t));
						}
						None => {}
					}
				}
			};
		}
	}
}

pub struct IterMut<'a, T> {
	base: std::slice::IterMut<'a, Slot<T>>,
	index: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
	type Item = (StoreIndex<T>, &'a mut T);

	fn next(&mut self) -> Option<(StoreIndex<T>, &'a mut T)> {
		loop {
			let base_next = self.base.next();
			match base_next {
				None => {
					return None;
				}
				Some(slot) => {
					let curr_idx = self.index;
					self.index += 1;
					match &mut slot.data {
						Some(t) => {
							return Some((StoreIndex::from_index(curr_idx), t));
						}
						None => {}
					}
				}
			};
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_add_idx() {
		let mut store: ObjectStore<String> = ObjectStore::new();
		let idx1 = store.add("foo".to_string());
		let idx2 = store.add("bar".to_string());
		assert_eq!(store.get(idx2), "bar");
		assert_eq!(store.get(idx1), "foo");
		assert_eq!(store.count(), 2);
		assert_eq!(store.size(), 2);
	}
	#[test]
	fn test_iter() {
		let mut store: ObjectStore<String> = ObjectStore::new();
		let idx1 = store.add("foo".to_string());
		let idx2 = store.add("bar".to_string());
		let idx3 = store.add("xyz".to_string());
		store.remove(idx2);
		assert_eq!(store.iter().count(), 2);
		assert!(store.iter().nth(0) == Some((idx1, &"foo".to_string())));
		assert!(store.iter().nth(1) == Some((idx3, &"xyz".to_string())));
		assert_eq!(store.count(), 2);
		assert_eq!(store.size(), 3);
	}
	#[test]
	fn test_remove_add() {
		let mut store: ObjectStore<String> = ObjectStore::new();
		store.add("foo".to_string());
		let idx2 = store.add("bar".to_string());
		store.add("xyz".to_string());
		store.remove(idx2);
		store.add("nya".to_string());
		assert_eq!(store.iter().count(), 3);
		assert_eq!(store.count(), 3);
		assert_eq!(store.size(), 3);
	}
	#[test]
	fn test_iter_mut() {
		let mut store: ObjectStore<String> = ObjectStore::new();
		store.add("foo".to_string());
		let idx2 = store.add("bar".to_string());
		*store.iter_mut().nth(1).unwrap().1 = "xyz".to_string();
		assert_eq!(store.get(idx2), "xyz");
	}
}
