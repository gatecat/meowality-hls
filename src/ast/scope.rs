use crate::ast::Statement;
use crate::core::IdString;

pub enum IdentifierType {
	Type,
	Func,
	Var,
}

pub struct ScopeLevel<'a> {
	pub parent: Option<&'a ScopeLevel<'a>>,
}