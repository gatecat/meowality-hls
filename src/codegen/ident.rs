use crate::core::{SSOArray, IdString};

// Resolved identifiers for uniquely refering to objects during elaboration
#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum IdentPart {
	Str(IdString), // named scopes use their name
	Anon(usize), // anonymous scopes use a scope serial number
	Invalid, // used for identifiers we should never see
}

impl Default for IdentPart {
	fn default() -> Self { IdentPart::Invalid }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Identifier {
	pub parts: SSOArray<IdentPart, 4>,
}