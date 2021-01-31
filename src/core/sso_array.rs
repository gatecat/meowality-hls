use std::ops::{Deref, DerefMut};

// This is an array type that preallocates Nsmall entries without needing heap; larger values go onto the heap
#[derive(Clone)]
pub enum SSOArray <T: Clone+Default+Copy, const N_SMALL: usize> {
	Small(usize, [T; N_SMALL]),
	Large(Vec<T>),
}

impl <T: Clone + Default + Copy, const N_SMALL: usize> SSOArray<T, N_SMALL> {
	pub fn new(len: usize) -> SSOArray<T, N_SMALL> {
		if len < N_SMALL {
			SSOArray::Small(len, [T::default(); N_SMALL])
		} else {
			SSOArray::Large(vec![T::default(); len])
		}
	}
	pub fn from_slice(sl: &[T]) -> SSOArray<T, N_SMALL> {
		if sl.len() < N_SMALL {
			let mut arr = [T::default(); N_SMALL];
			for (i, val) in sl.iter().enumerate() {
				arr[i] = val.clone();
			}
			SSOArray::Small(sl.len(), arr)
		} else {
			SSOArray::Large(Vec::from(sl))
		}
	}
	pub fn len(&self) -> usize {
		match self {
			SSOArray::Small(len, _) => *len,
			SSOArray::Large(v) => v.len(),
		}
	}
	pub fn push(&mut self, val: T) {
		match self {
			SSOArray::Small(len, arr) => {
				if *len+1 < N_SMALL {
					arr[*len] = val;
					*len += 1;
				} else {
					let mut next = Vec::from(&arr[0..*len]);
					next.push(val);
					*self = SSOArray::Large(next);
				}
			},
			SSOArray::Large(v) => {
				v.push(val);
			}
		}
	}
}

impl <T: Clone + Default + Copy, const N_SMALL: usize> Deref for SSOArray<T, N_SMALL> {
	type Target = [T];
	fn deref(&self) -> &[T] {
		match self {
			SSOArray::Small(len, arr) => &arr[0..*len],
			SSOArray::Large(v) => &v,
		}
	}
}

impl <T: Clone + Default + Copy, const N_SMALL: usize> DerefMut for SSOArray<T, N_SMALL> {
	fn deref_mut(&mut self) -> &mut [T] {
		match self {
			SSOArray::Small(len, arr) => &mut arr[0..*len],
			SSOArray::Large(v) => v,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	pub fn small() {
		let mut s = SSOArray::<u32, 5>::from_slice(&[1, 2, 3]);
		assert_eq!(s.len(), 3);
		assert_eq!(s[0], 1);
		assert_eq!(s.iter().nth(2).cloned(), Some(3));
		s[1] = 32;
		assert_eq!(s[1], 32);
	}
	#[test]
	pub fn large() {
		let mut s = SSOArray::<u32, 5>::from_slice(&[1, 2, 3, 4, 5, 6, 7]);
		assert_eq!(s.len(), 7);
		assert_eq!(s[0], 1);
		assert_eq!(s[6], 7);
		assert_eq!(s.iter().nth(2).cloned(), Some(3));
		s[5] = 32;
		assert_eq!(s[5], 32);
	}
	#[test]
	pub fn push() {
		let mut s = SSOArray::<u32, 5>::from_slice(&[1, 2, 3, 4]);
		assert_eq!(s.len(), 4);
		s.push(42);
		assert_eq!(s.len(), 5);
		assert_eq!(s[4], 42);
		s.push(69);
		assert_eq!(s.len(), 6);
		assert_eq!(s[5], 69);
	}
}