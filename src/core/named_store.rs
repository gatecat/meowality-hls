use crate::core::{IdStringDb, IdString};
use crate::core::NullValue;
use crate::core::object_store as base;
use crate::core::{ObjectStore, StoreIndex};
use rustc_hash::FxHashMap;

/*
This represents an item in a NamedStore; which always has a name and index associated with it
*/
pub trait NamedItem {
	fn get_name(&self) -> IdString;
	fn set_name(&mut self, new_name: IdString);
	fn set_index(&mut self, idx: StoreIndex<Self>);
}

/*
This is an extension of object_store; where objects can be accessed name or index
*/

pub struct NamedStore<T: NamedItem> {
	names: FxHashMap<IdString, StoreIndex<T>>,
	objects: base::ObjectStore<T>,
}

impl <T: NamedItem> NamedStore<T> {
	pub fn new() -> NamedStore<T> {
		NamedStore {
			names: FxHashMap::default(),
			objects: ObjectStore::new(),
		}
	}
	pub fn add(&mut self, obj: T) -> Result<StoreIndex<T>, String> {
		// Add an item to the store; setting its name and returning its index in the store
		let name = obj.get_name();
		if self.names.contains_key(&name) {
			Err(format!(
				"object named {} already exists",
				name
			))
		} else {
			let index = self.objects.add(obj);
			self.objects.get_mut(index).set_index(index);
			self.names.insert(name, index);
			Ok(index)
		}
	}
	pub fn get(&self, idx: StoreIndex<T>) -> &T {
		self.objects.get(idx)
	}
	pub fn get_mut(&mut self, idx: StoreIndex<T>) -> &mut T {
		self.objects.get_mut(idx)
	}
	pub fn named(&self, name: IdString) -> Option<&T> {
		self.names.get(&name).map(|&x| self.objects.get(x))
	}
	pub fn named_mut(&mut self, name: IdString) -> Option<&mut T> {
		match self.names.get(&name) {
			Some(&x) => Some(self.objects.get_mut(x)),
			None => None,
		}
	}
	pub fn rename(&mut self, old_name: IdString, new_name: IdString) -> Result<(), String> {
		let &idx = self.names.get(&old_name).ok_or(format!("no object named {}", old_name))?;
		self.get_mut(idx).set_name(new_name);
		self.names.remove(&old_name);
		if self.names.contains_key(&new_name) {
			return Err(format!("object named {} already exists", new_name));
		}
		self.names.insert(new_name, idx);
		Ok(())
	}
	pub fn remove(&mut self, idx: StoreIndex<T>) {
		self.objects.remove(idx);
	}
	pub fn remove_named(&mut self, name: IdString) {
		self.objects.remove(*self.names.get(&name).unwrap());
	}
	pub fn count(&self) -> usize {
		self.objects.count()
	}
	pub fn size(&self) -> usize {
		self.objects.size()
	}
	pub fn iter<'a>(&'a self) -> impl Iterator<Item=(StoreIndex<T>, &'a T)> {
		self.objects.iter()
	}
}



#[cfg(test)]
mod test {
	use super::*;

	struct TestObject {
		pub name: IdString,
		pub index: StoreIndex<TestObject>,
		pub value: u32
	}

	impl TestObject {
		pub fn new(name: IdString, value: u32) -> TestObject {
			TestObject {
				name: name,
				index: StoreIndex::NULL,
				value: value,
			}
		}
	}

	impl NamedItem for TestObject {
		fn get_name(&self) -> IdString { self.name }
		fn set_name(&mut self, new_name: IdString) { self.name = new_name }
		fn set_index(&mut self, index: StoreIndex<TestObject>) { self.index = index; }
	}

	#[test]
	fn add_get() {
		let mut ids_db = IdStringDb::new();
		let id_foo = ids_db.id("foo");
		let id_bar = ids_db.id("bar");

		let mut store : NamedStore<TestObject> = NamedStore::new();
		let idx_foo = store.add(TestObject::new(id_foo, 1)).unwrap();
		let idx_bar = store.add(TestObject::new(id_bar, 2)).unwrap();

		assert_eq!(store.add(TestObject::new(id_bar, 3)).unwrap_err(), "object named $(IdString-2) already exists".to_string());

		assert_eq!(store.get(idx_foo).index.index(), idx_foo.index());
		assert_eq!(store.get(idx_foo).value, 1);
		assert_eq!(store.get(idx_bar).value, 2);
	}

	#[test]
	fn named() {
		let mut ids_db = IdStringDb::new();
		let id_foo = ids_db.id("foo");
		let id_bar = ids_db.id("bar");
		let id_xyz = ids_db.id("xyz");

		let mut store : NamedStore<TestObject> = NamedStore::new();
		store.add(TestObject::new(id_foo, 1)).unwrap();
		store.add(TestObject::new(id_bar, 2)).unwrap();

		assert_eq!(store.named(id_foo).unwrap().value, 1);
		assert_eq!(store.named(id_bar).unwrap().value, 2);
		assert!(store.named(id_xyz).is_none());
	}

	#[test]
	fn mutable() {
		let mut ids_db = IdStringDb::new();
		let id_foo = ids_db.id("foo");
		let id_bar = ids_db.id("bar");

		let mut store : NamedStore<TestObject> = NamedStore::new();
		let idx_foo = store.add(TestObject::new(id_foo, 1)).unwrap();
		let idx_bar = store.add(TestObject::new(id_bar, 2)).unwrap();

		store.get_mut(idx_foo).value = 5;
		store.named_mut(id_bar).unwrap().value = 23;

		assert_eq!(store.get(idx_foo).value, 5);
		assert_eq!(store.get(idx_bar).value, 23);
	}

	#[test]
	fn renaming() {
		let mut ids_db = IdStringDb::new();
		let id_foo = ids_db.id("foo");
		let id_bar = ids_db.id("bar");

		let mut store : NamedStore<TestObject> = NamedStore::new();
		store.add(TestObject::new(id_foo, 1)).unwrap();
		store.rename(id_foo, id_bar).unwrap();
		assert_eq!(store.named(id_bar).unwrap().value, 1);
		assert!(store.named(id_foo).is_none());
	}
}
