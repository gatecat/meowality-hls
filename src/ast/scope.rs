use crate::ast::Statement;
use crate::core::IdString;

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
	// Gets a list of statements that could declare types, functions or variables in this scope
	fn get_decls<'a>(&'a self) -> Vec<&'a Statement>;
}