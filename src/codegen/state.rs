use crate::ast::{SrcInfo, Function};
use crate::core::{BitVector, State, StoreIndex, IdString, IdStringDb, ObjectStore, NullableIndex, constids};
use rustc_hash::FxHashMap;

use crate::design::{Node, Design, PrimitiveType};

use crate::codegen::*;

#[derive(Eq, PartialEq, Debug)]
pub struct CodegenError(pub SrcInfo, pub String);

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
	// The current stack of scopes
	scopes: Vec<GenScope>,
	// The current stack of conditionals to be applied by the evaluator
	conds: Vec<(StoreIndex<Node>, bool)>,
	auto_idx: usize,
}

impl <'a> GenState<'a> {
	pub fn new(ids: &'a mut IdStringDb, name: IdString) -> Self {
		Self {
			ids: ids,
			vars: ObjectStore::new(),
			funcs: FxHashMap::default(),
			structs: FxHashMap::default(),
			des: Design::new(name),
			scopes: vec![GenScope::new(0)],
			conds: Vec::new(),
			auto_idx: 0,
		}
	} 
	pub fn next_name(&mut self, base: IdString) -> IdString {
		self.auto_idx += 1;
		let new_name = format!("{}${}$", self.ids.get_str(base), self.auto_idx);
		self.ids.id(&new_name)
	}
	pub fn get_node(&mut self, value: &RValue, src: SrcInfo) -> StoreIndex<Node> {
		match value {
			RValue::Node(n) => *n,
			RValue::Constant(c) => self.des.add_const(self.ids, c.clone(), src),
			_ => unreachable!(),
		}
	}
	pub fn apply_conditionals(&mut self, base_name: IdString, old_value: RValue, new_value: RValue, src: SrcInfo) -> RValue {
		if self.conds.is_empty() {
			return new_value;
		}
		let mut cond_inv = BitVector::new(self.conds.len(), false);
		for (i, (_, inv)) in self.conds.iter().enumerate() {
			cond_inv.set(i, if *inv { State::S1 } else { State::S0 });
		}
		let prim_name = self.next_name(base_name);
		let prim = self.des.add_prim(prim_name, PrimitiveType::Cond { inv: cond_inv}, src).unwrap();
		let old_node = self.get_node(&old_value, src);
		let new_node = self.get_node(&new_value, src);
		self.des.add_prim_input(prim, constids::A, old_node).unwrap();
		self.des.add_prim_input(prim, constids::B, new_node).unwrap();
		for (i, (node, _)) in self.conds.iter().enumerate() {
			let port_name = self.ids.id(&format!("S{}", i));
			self.des.add_prim_input(prim, port_name, *node).unwrap();
		}
		let typ = self.des.nodes.get(new_node).typ;
		let node_name = self.next_name(base_name);
		RValue::from_node(self.des.add_node(node_name, typ, src, prim, constids::Q).unwrap())
	}
	pub fn assign_variable(&mut self, var: StoreIndex<Variable>, path: &[ValuePathItem], new_value: &RValue, src: SrcInfo) {
		let curr_value = self.vars.get(var).value.get(path).clone();
		if curr_value.is_scalar() {
			// at the end of the line, actually assign the value
			assert!(new_value.is_scalar());
			let value_name = self.vars.get(var).name;
			let applied_value = self.apply_conditionals(value_name, curr_value.clone(), new_value.clone(), src);
			self.vars.get_mut(var).value.set(path, applied_value);
		} else {
			match new_value {
				RValue::Array(values) => {
					for (i, val) in values.iter().enumerate() {
						let mut next_path = Vec::from(path);
						next_path.push(ValuePathItem::Index(i));
						self.assign_variable(var, &next_path, val, src);
					}
				},
				RValue::Structure(sv) => {
					for (key, val) in sv.values.iter() {
						let mut next_path = Vec::from(path);
						next_path.push(ValuePathItem::Member(*key));
						self.assign_variable(var, &next_path, val, src);
					}
				}
				_ => unreachable!(),
			}
		}
	}
	pub fn scope(&mut self) -> &mut GenScope {
		self.scopes.last_mut().unwrap()
	}
	pub fn push_scope(&mut self) {
		self.scopes.push(GenScope::new(self.conds.len()));
	}
	pub fn pop_scope(&mut self) {
		self.scopes.pop();
	}
	pub fn push_cond(&mut self, cond: StoreIndex<Node>, invert: bool) {
		self.conds.push((cond, invert))
	}
	pub fn pop_cond(&mut self) {
		self.conds.pop();
	}
	pub fn lookup_var(&self, ident: IdString) -> NullableIndex<Variable> {
		for scope in self.scopes.iter().rev() {
			if let Some(var) = scope.var_map.get(&ident) {
				return NullableIndex::some(*var);
			}
		}
		return NullableIndex::none();
	}
	pub fn lookup_type(&self, ident: IdString) -> Option<&ResolvedType> {
		for scope in self.scopes.iter().rev() {
			if let Some(typ) = scope.type_map.get(&ident) {
				return Some(typ);
			}
		}
		return None;
	}
}

// Codegen state for a specific scope
pub struct GenScope {
	// Mapping from var names in the current scope to concrete variable indices
	pub var_map: FxHashMap<IdString, StoreIndex<Variable>>,
	// Mapping from type names in the current scope to resolved types
	pub type_map: FxHashMap<IdString, ResolvedType>,
	// Index into the condition stack where this scope starts
	pub cond_idx: usize,
}

impl GenScope {
	pub fn new(cond_idx: usize) -> GenScope {
		GenScope {
			var_map: FxHashMap::default(),
			type_map: FxHashMap::default(),
			cond_idx: cond_idx,
		}
	}
}