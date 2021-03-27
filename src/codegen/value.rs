use crate::ast::{DataType, StructureDef, Function};
use crate::core::{BitVector, StoreIndex, IdString};
use crate::design::Node;
use rustc_hash::FxHashMap;

// All variables are tracked this way
pub struct Variable {
	pub name: IdString,
	pub typ: DataType,
	pub value: Value,
}

// The contents of a structure
pub struct StructureValue {
	pub typ: StoreIndex<StructureDef>,
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