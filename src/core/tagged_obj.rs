// This is a tagged identifier that can be used to refer to any kind of object with full cross-referencing
// in various places; and can be created without any contextual information. Uses include the Tcl and logging
// APIs

// IDs are currently 64-bits long which is plenty to serialise anything in planar

use std::fmt;
use std::str::FromStr;
use crate::core::{IdString, StoreIndex};

// PackableIdent packs/unpacks identifiers to 64 contiguous bits
pub trait PackableIdent {
	fn pack_ident(&self) -> u64;
	fn unpack_ident(id: u64) -> Self;
}

// This should really be a proc-macro derive, but this works for now and is a bit simpler
macro_rules! impl_packable (
	($type:ident, $( $field_name:tt : $field_type:tt ),*) => {
		impl PackableIdent for $type {
			fn pack_ident(&self) -> u64 {
				let mut result = 0;
				let mut offset = 0;
				$(
					{
						let bits = std::mem::size_of::<$field_type>() * 8;
						result |= (self.$field_name as u64) << offset;
						offset += bits;
						assert!(offset <= 64);
					}
				)*
				return result;
			}
			fn unpack_ident(id: u64) -> Self {
				let mut offset = 0;
				Self {
					$(
						$field_name: {
							let bits = std::mem::size_of::<$field_type>() * 8;
							let value = (id >> offset) as $field_type;
							offset += bits;
							assert!(offset <= 64);
							value
						},
					)*
				}
			}
		}
	};
);

impl_packable!(IdString, index:u32);

// Special case of PackableIdent for store indices
impl <T> PackableIdent for StoreIndex<T> {
	fn pack_ident(&self) -> u64 {
		self.index() as u64
	}
	fn unpack_ident(id: u64) -> Self {
		StoreIndex::from_index(id as usize)
	}
}

pub trait TaggableObj {
	const TYPE_TAG: ObjectTag;
}

macro_rules! object_types {
	($(($type:ident, $singular:expr, $plural: expr)),*,) => {
		#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
		pub enum ObjectTag {
			$($type),*
		}
		impl ObjectTag {
			pub fn name(&self) -> &'static str {
				match self {
					$(ObjectTag::$type => stringify!($type)),*
				}
			}
			pub fn from_name(s: &str) -> Option<ObjectTag> {
				match s {
					$(stringify!($type) => Some(ObjectTag::$type)),*,
					_ => None,
				}
			}
			pub fn singular(&self) -> &'static str {
				match self {
					$(ObjectTag::$type => $singular),*
				}
			}
			pub fn plural(&self) -> &'static str {
				match self {
					$(ObjectTag::$type => $plural),*
				}
			}
		}
		$(
			impl TaggableObj for $type {
			   const TYPE_TAG: ObjectTag = ObjectTag::$type;
			}

			// Our magic Display information uses encoded TaggableObj
			// to enable logging without full context info and cross-reference
			// generation downstream
			impl fmt::Display for $type {
				fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
					write!(f, "{}", TaggedObj::tag_obj(self))?;
					Ok(())
				}
			}

		)*
	};
}

object_types! {
	(IdString, "IdString", "IdStrings"),
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct TaggedObj {
	pub tag: ObjectTag,
	pub index: u64,
}

impl TaggedObj {
	pub fn tag_obj<T: TaggableObj + PackableIdent>(obj: &T) -> TaggedObj {
		TaggedObj {
			tag: T::TYPE_TAG,
			index: obj.pack_ident(),
		}
	}
	pub fn untag_obj<T: TaggableObj + PackableIdent>(&self) -> Option<T> {
		if self.tag == T::TYPE_TAG {
			Some(T::unpack_ident(self.index))
		} else {
			None
		}
	}
}

impl fmt::Display for TaggedObj {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// We can't convert an IdString to a concrete string at this stage as we don't have a reference to the context
		// So we insert a placeholder string instead; which the final stage of error reporting with context access will replace
		// This way we don't have to rewrite Rust's formatting infrastructure
		write!(f, "$({}-{:X})", self.tag.name(), self.index)?;
		Ok(())
	}
}

impl fmt::Debug for TaggedObj {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self)?;
		Ok(())
	}
}

impl FromStr for TaggedObj {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() < 4 {
			return Err(());
		}
		if &s[0..2] != "$(" || &s[s.len()-1..s.len()] != ")" {
			return Err(());
		}
		let sep_pos = s.find('-').ok_or(())?;
		Ok(TaggedObj {
			tag: ObjectTag::from_name(&s[2..sep_pos]).ok_or(())?,
			index: u64::from_str_radix(&s[sep_pos+1..s.len()-1], 16).or(Err(()))?,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;

	// PackableObj tests
	#[test]
	fn pack_idstring() {
		assert_eq!(IdString {index: 1}.pack_ident(), 1);
		assert_eq!(IdString {index: 0x12345678}.pack_ident(), 0x12345678);
	}
	#[test]
	fn unpack_idstring() {
		assert_eq!(IdString::unpack_ident(1), IdString {index: 1});
		assert_eq!(IdString::unpack_ident(0x12345678), IdString {index: 0x12345678});
	}
	// TaggedObj tests
	#[test]
	fn disp_tag() {
		assert_eq!(&format!("{}", TaggedObj {tag: ObjectTag::IdString, index: 1} ), "$(IdString-1)");
		assert_eq!(&format!("{}", TaggedObj {tag: ObjectTag::IdString, index: 0xFFFFFFFFFFFFFFFF} ), "$(IdString-FFFFFFFFFFFFFFFF)");
		assert_eq!(&format!("{}", TaggedObj {tag: ObjectTag::IdString, index: 0xFF00FFFFFFFFFF} ), "$(IdString-FF00FFFFFFFFFF)");
	}
	#[test]
	fn parse_tag() {
		assert_eq!("foobar".parse::<TaggedObj>(), Err(()));
		assert_eq!("$".parse::<TaggedObj>(), Err(()));
		assert_eq!("$(nothing)".parse::<TaggedObj>(), Err(()));
		assert_eq!("$(AType-1234)".parse::<TaggedObj>(), Err(()));
		assert_eq!("$(IdString-1)".parse::<TaggedObj>(), Ok(TaggedObj {tag: ObjectTag::IdString, index: 1}));
		assert_eq!("$(IdString-FFFFFFFFFFFFFFFF)".parse::<TaggedObj>(), Ok(TaggedObj {tag: ObjectTag::IdString, index: 0xFFFFFFFFFFFFFFFF}));
		assert_eq!("$(IdString-FF00FFFFFFFFFF)".parse::<TaggedObj>(), Ok(TaggedObj {tag: ObjectTag::IdString, index: 0xFF00FFFFFFFFFF}));
	}
	#[test]
	fn tag_disp() {
		assert_eq!(format!("{}", IdString {index: 1}), "$(IdString-1)");
	}
}
