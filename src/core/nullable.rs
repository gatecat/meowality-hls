// This is a zero-memory-overhead, approach to nullable index types that still provides type safety
// provided one 'invalid' value is available

pub trait NullValue {
	const NULL : Self;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Nullable<T: NullValue> {
	value: T,
}

impl <T: NullValue + PartialEq> Nullable<T> {
	#[inline]
	pub fn some(value: T) -> Nullable<T> {
		assert!(value != T::NULL);
		Nullable {value: value}
	}
	#[inline]
	pub fn none() -> Nullable<T> {
		Nullable {value: T::NULL}
	}
	#[inline]
	pub fn is_none(&self) -> bool {
		self.value == T::NULL
	}
	#[inline]
	pub fn is_some(&self) -> bool {
		self.value != T::NULL
	}
	#[inline]
	pub fn unwrap(self) -> T {
		assert!(self.value != T::NULL);
		self.value
	}
	#[inline]
	pub fn unwrap_or_err<U, F>(self, f: F) -> Result<T, U> where
		F: FnOnce() -> U {
		if self.value == T::NULL {
			Err(f())
		} else {
			Ok(self.value)
		}
	}
	#[inline]
	pub fn map<U, F>(self, f: F) -> Option<U> where
		F: FnOnce(T) -> U
	{
		if self.value == T::NULL {
			None
		} else {
			Some(f(self.value))
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[derive(PartialEq, Debug)]
	struct TestStruct(u32);

	impl NullValue for TestStruct { const NULL: TestStruct = TestStruct(0xAAAAAAAA); }

	#[test]
	fn basics() {
		let our_none = Nullable::<TestStruct>::none();
		assert!(our_none.is_none());
		assert!(!our_none.is_some());
		let our_some = Nullable::<TestStruct>::some(TestStruct(0x12345678));
		assert!(our_some.is_some());
		assert!(!our_some.is_none());
		assert_eq!(our_some.unwrap(), TestStruct(0x12345678));
	}

	#[test]
	#[should_panic(expected = "assertion failed: self.value != T::NULL")]
	fn unwrap_fail() {
		Nullable::<TestStruct>::none().unwrap();
	}

	#[test]
	#[should_panic(expected = "assertion failed: value != T::NULL")]
	fn some_null() {
		Nullable::<TestStruct>::some(TestStruct(0xAAAAAAAA));
	}

	#[test]
	fn map() {
		let our_some = Nullable::<TestStruct>::some(TestStruct(0x12345678));
		assert_eq!(our_some.map(|x| x.0 + 1).unwrap(), 0x12345679);
	}
}
