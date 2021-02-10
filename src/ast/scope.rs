use crate::ast::Statement;
use crate::core::IdString;

pub enum IdentifierType {
	Type,
	Func,
	Var,
}

pub trait Scope {
	// Get the name of the scope, if applicable
	fn name(&self) -> Option<IdString> {
		None
	}
	// Determine if an IdString names a data type in this scope
	fn is_type(&self, ident: IdString) -> bool;
	// Determine if an IdString names a function in this scope
	fn is_func(&self, ident: IdString) -> bool;
	// Determine if an IdString names a variable in this scope
	fn is_var(&self, ident: IdString) -> bool;
	// Get the type of an identifier, if valid
	fn lookup_ident(&self, ident: IdString) -> Option<IdentifierType> {
		if self.is_type(ident) {
			Some(IdentifierType::Type)
		} else if self.is_func(ident) {
			Some(IdentifierType::Func)
		} else if self.is_var(ident) {
			Some(IdentifierType::Var)
		} else {
			None
		}
	}
	// Gets a list of statements that could declare types, functions or variables in this scope
	fn get_decls<'a>(&'a self) -> Vec<&'a Statement>;
}