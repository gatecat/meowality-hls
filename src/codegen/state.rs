use crate::ast::{Function};
use crate::core::{BitVector, State, StoreIndex, IdString, IdStringDb, ObjectStore, NullableIndex, constids};
use rustc_hash::FxHashMap;

use crate::design::{Node, Design, PrimitiveType};

use crate::codegen::*;

// Codegen state for the elaboration of a module
pub struct GenState<'a> {
	// The current IdString database
	pub ids: &'a mut IdStringDb,
	// A list of all variables in all scopes - the scopes themselves store pointers to hese
	pub vars: ObjectStore<Variable>,
	// A list of all derived (post-template-substitution) functions and structures
	pub funcs: FxHashMap<ResolvedKey, Function>,
	pub structs: FxHashMap<ResolvedKey, DerivedStruct>,
	// The elaborated design
	pub des: Design,
	// The current stack of conditionals to be applied by the evaluator
	conds: Vec<(StoreIndex<Node>, bool)>,
	auto_idx: usize,
}

impl <'a> GenState<'a> {
	fn next_name(&mut self, base: IdString) -> IdString {
		self.auto_idx += 1;
		let new_name = format!("{}${}$", self.ids.get_str(base), self.auto_idx);
		self.ids.id(&new_name)
	}
	fn apply_conditionals(&mut self, base_name: IdString, old_value: StoreIndex<Node>, new_value: StoreIndex<Node>) -> StoreIndex<Node> {
		if self.conds.is_empty() {
			return new_value;
		}
		let mut cond_inv = BitVector::new(self.conds.len(), false);
		for (i, (_, inv)) in self.conds.iter().enumerate() {
			cond_inv.set(i, if *inv { State::S1 } else { State::S0 });
		}
		let prim_name = self.next_name(base_name);
		let prim = self.des.add_prim(prim_name, PrimitiveType::Cond { inv: cond_inv}).unwrap();
		self.des.add_prim_input(prim, constids::A, old_value).unwrap();
		self.des.add_prim_input(prim, constids::B, new_value).unwrap();
		for (i, (node, _)) in self.conds.iter().enumerate() {
			let port_name = self.ids.id(&format!("S{}", i));
			self.des.add_prim_input(prim, port_name, *node).unwrap();
		}
		let typ = self.des.nodes.get(new_value).typ;
		let node_name = self.next_name(base_name);
		self.des.add_node(node_name, typ, prim, constids::Q).unwrap()
	}
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