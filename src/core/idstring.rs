use std::fmt;
use std::convert::TryInto;
use crate::core::{IndexedSet, ToStrWithCtx};
use crate::design::Context;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct IdString { pub index: u32 }

// Our fast indexed string pool

impl IdString {
	pub const NONE : IdString = IdString { index: 0};
	pub fn val(&self) -> u32 {
		return self.index;
	}
	pub fn str<'a>(&self, ids: &'a IdStringDb) -> &'a str {
		ids.get_str(*self)
	}
}

impl fmt::Debug for IdString {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// We can't convert an IdString to a concrete string at this stage as we don't have a reference to the context
		// So we insert a placeholder string instead; which the final stage of error reporting with context access will replace
		// This way we don't have to rewrite Rust's formatting infrastructure
		write!(f, "`{}`", self.index)
	}
}

impl ToStrWithCtx for IdString {
	fn to_str_with_ctx(&self, ctx: &Context) -> String {
		self.str(&ctx.ids).to_string()
	}
}

pub struct IdStringDb {
	ids: IndexedSet<String>
}

impl IdStringDb {
	pub fn new() -> IdStringDb {
		let mut set = IndexedSet::new();
		set.add(&"".to_string()); // 0 is always the empty string
		IdStringDb { ids: set }
	}
	pub fn id(&mut self, s: &str) -> IdString {
		IdString{index: self.ids.add(&s.to_string()).try_into().unwrap()}
	}
	pub fn get_id(&self, s: &str) -> Option<IdString> {
		self.ids.get_index(&s.to_string()).map(|i| IdString{ index: i.try_into().unwrap() })
	}
	pub fn get_str(&self, i: IdString) -> &str {
		self.ids.key(i.index.try_into().unwrap())
	}
	pub fn init_add(&mut self, s: &str, i: u32) {
		let index = self.ids.add(&s.to_string());
		assert_eq!(index, i.try_into().unwrap());
	}
}

pub fn strip_raw_prefix<'a>(s: &'a str) -> &'a str {
	if s.starts_with("r#") {
		&s[2..]
	} else {
		s
	}
}

#[macro_export] macro_rules! constids {
	($($id:tt),*,) => {
		#[repr(u32)]
		#[allow(non_camel_case_types)]
		enum ConstIdIdx {
			ID_NONE = 0,
			$($id),*
		}
		$(#[allow(non_upper_case_globals)] pub const $id : crate::core::IdString = crate::core::idstring::IdString { index: ConstIdIdx::$id as u32 };)*
		pub fn do_ids_init(db: &mut crate::core::IdStringDb) {
			$(db.init_add(crate::core::idstring::strip_raw_prefix(stringify!($id)), ConstIdIdx::$id as u32));*
		}
	};
}
#[cfg(test)]
mod test_constids {
	constids! {
		LUT4,
		DFF,
		IO,
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn basics() {
		let mut db = IdStringDb::new();
		let id_foo = db.id("foo");
		let id_bar = db.id("bar");
		assert_eq!(id_foo.str(&db), "foo");
		assert_eq!(id_bar.str(&db), "bar");
		assert_eq!(db.get_id("foo"), Some(id_foo));
		assert_eq!(db.get_id("xyz"), None);
	}
	#[test]
	fn constid() {
		let mut db = IdStringDb::new();
		test_constids::do_ids_init(&mut db);
		assert_eq!(test_constids::LUT4.str(&db), "LUT4");
		assert_eq!(test_constids::DFF.str(&db), "DFF");
		assert_eq!(db.id("IO"), test_constids::IO);
	}
}
