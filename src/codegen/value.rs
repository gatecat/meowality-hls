use std::fmt;
use crate::codegen::{State, ResolvedType, ResolvedTypes, ResolvedKey};
use crate::ast::{Function};
use crate::core::{BitVector, StoreIndex, IdString};
use crate::design::Node;
use rustc_hash::FxHashMap;

// All variables are tracked this way
pub struct Variable {
	pub name: IdString,
	pub typ: ResolvedType,
	pub value: Value,
}

// The contents of a structure
pub struct StructureValue {
	pub typ: ResolvedKey,
	pub values: FxHashMap<IdString, Value>,
}

// Lots of different things can be 'values' in our codegen IL
pub enum Value {
	Void, // void type, probably shouldn't exist
	Constant(BitVector), // a resolved constant value
	Node(StoreIndex<Node>), // variables become pointers to nodes in the design being elaborated
	Structure(StructureValue), // a structure, stored as the structure type and name-value map
	Array(Vec<Value>), // an array, stored as a list of values
	Func(StoreIndex<Function>), // a function 'pointer'
	Ref(StoreIndex<Variable>), // a reference to another variable (TODO: what are we actually indexing) 
}

impl Value {
	// Create an outline value from a resolved type (with leaf values filled with Void)
	pub fn from_type(st: &State, ty: &ResolvedTypes) -> Value {
		use Value::*;
		match ty {
			ResolvedTypes::Integer(it) => Constant(BitVector::undefined(it.width, it.is_signed)),
			ResolvedTypes::Struct(key) => {
				let struct_data = st.structs.get(key).unwrap();
				Structure(StructureValue {
					typ: key.clone(),
					values: struct_data.members.iter().map(|(k, t)| (*k, Self::from_type(st, &t.typ))).collect(),
				})
			}
			ResolvedTypes::Array(base, count) => Array((0..*count).map(|_| Self::from_type(st, &base.typ)).collect()),
			ResolvedTypes::Reference(_) => unimplemented!(), // special_case
			_ => Void,
		}
	}
	// Gets the type of a value
	pub fn to_type(&self, st: &State) -> Option<ResolvedType> {
		Some(ResolvedType {
			is_const: true,
			is_static: false,
			typ: match self {
				Value::Void => Some(ResolvedTypes::Void),
				Value::Constant(bv) => Some(ResolvedTypes::Integer(bv.op_type())),
				Value::Node(n) => Some(ResolvedTypes::Integer(st.des.nodes.get(*n).typ)),
				Value::Structure(sv) => Some(ResolvedTypes::Struct(sv.typ.clone())),	
				Value::Array(vals) => {
					let mut typ = ResolvedType { is_const: true, is_static: false, typ: ResolvedTypes::Void };
					for val in vals.iter() {
						typ = typ.merge(&val.to_type(st)?)?;
					}
					Some(typ.typ)
				},
				Value::Ref(v) => {
					Some(ResolvedTypes::Reference(Box::new(st.vars.get(*v).typ.clone())))
				}
				_ => unimplemented!(),
			}?
		})
	}
}

impl fmt::Debug for Value {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		use Value::*;
		match self {
			Void => write!(fmt, "<void>")?,
			Constant(v) => write!(fmt, "{:?}", v)?,
			Node(n) => write!(fmt, "{:?}", n)?,
			_ => unimplemented!(),
		}
		Ok(())
	}
}