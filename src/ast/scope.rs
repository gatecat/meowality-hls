use crate::ast::Statement;
use crate::core::IdString;

pub enum IdentifierType {
	Type,
	Var,
}

// Various different kinds of scopes; from the parser point of view
pub trait ScopeEntry {
	fn is_type(&self, id: IdString) -> bool;
	fn is_var(&self, id: IdString) -> bool;
}

pub struct NullEntry;

impl ScopeEntry for NullEntry {
	fn is_type(&self, _: IdString) -> bool { false }
	fn is_var(&self, _: IdString) -> bool { false }
}

pub struct StructHeaderEntry {
	pub name: IdString,
}

impl ScopeEntry for StructHeaderEntry {
	fn is_type(&self, id: IdString) -> bool { id == self.name }
	fn is_var(&self, _: IdString) -> bool { false }
}

impl ScopeEntry for Vec<Statement> {
	fn is_type(&self, id: IdString) -> bool { self.iter().any(|st| st.leaf_is_type(id)) }
	fn is_var(&self, id: IdString) -> bool { self.iter().any(|st| st.leaf_is_var(id)) }
}


pub struct ScopeLevel<'a> {
	pub parent: Option<&'a ScopeLevel<'a>>,
	pub entry: &'a dyn ScopeEntry,
}