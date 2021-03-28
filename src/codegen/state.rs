use crate::ast::{StructureDef, Function};
use crate::core::{StoreIndex, IdString, ObjectStore};
use rustc_hash::FxHashMap;

use crate::codegen::*;

// Codegen state for the elaboration of a module
pub struct State {
	// A list of all variables in all scopes - the scopes themselves store pointers to hese
	pub vars: ObjectStore<Variable>,
	// A list of all derived (post-template-substitution) functions and structures
	pub funcs: FxHashMap<ResolvedKey, Function>,
	pub structs: FxHashMap<ResolvedKey, StructureDef>,
}

// Codegen state for a specific scope
pub struct GenScope {
	// Mapping from var names in the current scope to concrete variable indices
	pub var_map: FxHashMap<IdString, StoreIndex<Variable>>,
	// Mapping from type names in the current scope to resolved types
	pub type_map: FxHashMap<IdString, ResolvedType>,
}