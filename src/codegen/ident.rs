use crate::core::{SSOArray, IdString};
use std::fmt;

// Resolved identifiers for uniquely refering to objects during elaboration
#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum IdentPart {
	Str(IdString), // named scopes use their name
	Anon(usize), // anonymous scopes use a scope serial number
	Invalid, // used for identifiers we should never see
}

impl fmt::Debug for IdentPart {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		match &self {
			IdentPart::Str(s) => write!(fmt, "{:?}", s)?,
			IdentPart::Anon(_) => write!(fmt, "<anonymous>")?,
			IdentPart::Invalid => write!(fmt, "<invalid>")?,
		}
		Ok(())
	}
}

impl Default for IdentPart {
	fn default() -> Self { IdentPart::Invalid }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Identifier {
	pub parts: SSOArray<IdentPart, 4>,
}

impl fmt::Debug for Identifier {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		for (i, part) in self.parts.iter().enumerate() {
			if i > 0 { write!(fmt, "::")?; }
			write!(fmt, "{:?}", part)?;
		}
		Ok(())
	}
}