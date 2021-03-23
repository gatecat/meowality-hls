use crate::core::{BasicOp, BitVector, Constant, IdString, NamedItem, NamedStore, StoreIndex};
use crate::design::{PortRef, Node};
use rustc_hash::FxHashMap;

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

// Memory primitives
pub struct Memory {
	pub width: usize,
	pub depth: usize,
	pub read_ports: usize,
	pub write_ports: usize,
	pub is_external: bool,
	pub init: Vec<BitVector>,
}

pub enum PrimitiveType {
	Constant(BitVector),
	BasicOp(BasicOp),
	SpecOp(SpecialOperation),
	Reg(Register),
	Mem(Memory),
	TopPort,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PortDir {
	Input,
	Output,
}

// A primitive port
pub struct PrimitivePort {
	pub name: IdString,
	pub index: StoreIndex<PrimitivePort>,
	pub dir: PortDir,
	pub node: StoreIndex<Node>,
	pub usr_idx: Option<StoreIndex<PortRef>>,
}

impl NamedItem for PrimitivePort {
	fn get_name(&self) -> IdString { self.name }
	fn set_name(&mut self, name: IdString) { self.name = name; }
	fn set_index(&mut self, index: StoreIndex<Self>) { self.index = index; }
}

// An instance of a primitive
pub struct Primitive {
	pub name: IdString,
	pub index: StoreIndex<Primitive>,
	pub typ: PrimitiveType,
	pub attrs: FxHashMap<IdString, Constant>,
	pub ports: NamedStore<PrimitivePort>,
}

impl NamedItem for Primitive {
	fn get_name(&self) -> IdString { self.name }
	fn set_name(&mut self, name: IdString) { self.name = name; }
	fn set_index(&mut self, index: StoreIndex<Self>) { self.index = index; }
}