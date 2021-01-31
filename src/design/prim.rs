use crate::core::{BitVector, Constant, IdString, StoreIndex};
use crate::design::Node;
use rustc_hash::FxHashMap;

// The standard C type operations
pub enum BasicOperation {
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Neg,
	Eq,
	Neq,
	Gt,
	Lt,
	GtEq,
	LtEq,
	Shl,
	Shr,
	BwAnd,
	BwOr,
	BwXor,
	BwNot,
	LogAnd,
	LogOr,
	LogNot,
	LogCast,
}

// Special operations for certain hardware-y things
pub enum SpecialOperation {
	Mux(usize), // for conditionals, array reads, etc
	SetIfEq{pattern: BitVector}, // for array writes; replace value if index matches
	SliceGetFix{offset: usize, width: usize}, // compile time bitslice extraction 
	SliceGetVar{step: usize, width: usize}, // run time bitslice extraction
	SliceSetFix{offset: usize, width: usize}, // compile time bitslice replacement
	SliceSetVar{step: usize, width: usize}, // run time bitslice replacement
}

// The various kinds of registers we use
pub enum Register {
	Delay(usize), // a deliberately created delay of K cycles
	Storage, // storing values between runs
	Pipeline, // a register inserted to meet timing
}

pub enum PrimitiveType {
	BasicOp(BasicOperation),
	SpecOp(SpecialOperation),
	Reg(Register),
}

// An instance of a primitive
pub struct Primitive {
	pub name: IdString,
	pub typ: PrimitiveType,
	pub attrs: FxHashMap<IdString, Constant>,
	pub inputs: FxHashMap<IdString, StoreIndex<Node>>,
	pub outputs: FxHashMap<IdString, StoreIndex<Node>>,
}