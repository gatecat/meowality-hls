use crate::ast::{Function};
use crate::core::{StoreIndex, IdString, ObjectStore, NullableIndex};
use rustc_hash::FxHashMap;

use crate::design::Design;

use crate::codegen::*;

// Codegen state for the elaboration of a module
pub struct State {
	// A list of all variables in all scopes - the scopes themselves store pointers to hese
	pub vars: ObjectStore<Variable>,
	// A list of all derived (post-template-substitution) functions and structures
	pub funcs: FxHashMap<ResolvedKey, Function>,
	pub structs: FxHashMap<ResolvedKey, DerivedStruct>,
	// The elaborated design
	pub des: Design,
}

// Codegen state for a specific scope
pub struct GenScope<'a> {
	pub parent_scope: Option<&'a GenScope<'a>>,
	// Mapping from var names in the current scope to concrete variable indices
	pub var_map: FxHashMap<IdString, StoreIndex<Variable>>,
	// Mapping from type names in the current scope to resolved types
	pub type_map: FxHashMap<IdString, ResolvedType>,
}

impl <'a> GenScope<'a> {
	pub fn new(parent_scope: Option<&'a GenScope<'a>>) -> GenScope<'a> {
		GenScope {
			parent_scope: parent_scope,
			var_map: FxHashMap::default(),
			type_map: FxHashMap::default(),
		}
	}
	pub fn lookup_var(&self, ident: IdString) -> NullableIndex<Variable> {
		if let Some(var) = self.var_map.get(&ident) {
			NullableIndex::some(*var)
		} else if let Some(parent) = self.parent_scope {
			parent.lookup_var(ident)
		} else {
			NullableIndex::none()
		}
	}
	pub fn lookup_type(&self, ident: IdString) -> Option<&ResolvedType> {
		if let Some(typ) = self.type_map.get(&ident) {
			Some(typ)
		} else if let Some(parent) = self.parent_scope {
			parent.lookup_type(ident)
		} else {
			None
		}
	}
}