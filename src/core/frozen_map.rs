use std::cmp::Ord;

pub trait GetKey {
	type Key;
	fn key(&self) -> Self::Key;
}

pub fn lookup<'a, T: GetKey>(list: &'a [T], key: &T::Key) -> Option<&'a T> where T::Key: Ord {
	use std::cmp::Ordering;
	if list.len() != 0 {
		let mut begin = 0;
		let mut end = list.len() - 1;
		while begin <= end {
			let i = (begin + end) / 2;
			match list[i].key().cmp(key) {
				Ordering::Equal => return Some(&list[i]),
				Ordering::Greater => { end = i - 1; } 
				Ordering::Less => { begin = i + 1; }
			}
		}
	}

	None
}

pub fn prepare<T:GetKey>(list: &mut [T]) where T::Key: Ord {
	list.sort_by(|a, b| a.key().cmp(&b.key()));
}


#[cfg(test)]
mod test {
	use super::*;
	struct MapObj {
		from: u32,
		to: u32,
		value: u32,
	}
	impl GetKey for MapObj {
		type Key = (u32, u32);
		fn key(&self) -> Self::Key { (self.from, self.to) }
	}
	#[test]
	fn basics() {
		let mut objects: Vec<MapObj> = Vec::new();
		objects.push(MapObj { from: 13, to: 37, value: 42});
		objects.push(MapObj { from: 0, to: 0, value: 123});
		objects.push(MapObj { from: 101, to: 101, value: 456});
		prepare(&mut objects);
		assert_eq!(lookup(&objects, &(101, 101)).unwrap().value, 456);
		assert_eq!(lookup(&objects, &(13, 37)).unwrap().value, 42);
		assert_eq!(lookup(&objects, &(0, 0)).unwrap().value, 123);
		assert!(lookup(&objects, &(0, 1)).is_none());
	}
}