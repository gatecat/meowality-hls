use crate::ast::SrcInfo;
use crate::core::{BasicOp, BitVector, Constant, IdString, NamedItem, NamedStore, NullableIndex, StoreIndex};
use crate::design::{PortRef, Node};
use rustc_hash::FxHashMap;

// Special operations for certain hardware-y things
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecialOperation {
	Mux(usize), // for array reads, etc
	SetIfEq{pattern: BitVector}, // for array writes; replace value if index matches
	SliceGetFix{offset: usize, width: usize}, // compile time bitslice extraction 
	SliceGetVar{step: usize, width: usize}, // run time bitslice extraction
	SliceSetFix{offset: usize, width: usize}, // compile time bitslice replacement
	SliceSetVar{step: usize, width: usize}, // run time bitslice replacement
}

// The various kinds of registers we use
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Register {
	Delay(usize), // a deliberately created delay of K cycles
	Storage, // storing values between runs
	Pipeline, // a register inserted to meet timing
}

// Memory primitives
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Memory {
	pub width: usize,
	pub depth: usize,
	pub read_ports: usize,
	pub write_ports: usize,
	pub is_external: bool,
	pub init: Vec<BitVector>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrimitiveType {
	Constant(BitVector),
	BasicOp(BasicOp),
	SpecOp(SpecialOperation),
	Cond{ inv: BitVector }, // conditional tree entry, width of mask decides condition input count and 1 in mask indicates inversion
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
	pub index: NullableIndex<Self>,
	pub dir: PortDir,
	pub node: NullableIndex<Node>,
	pub usr_idx: NullableIndex<PortRef>,
}

impl PrimitivePort {
	pub fn input(name: IdString, node: StoreIndex<Node>, usr_idx: StoreIndex<PortRef>) -> PrimitivePort {
		PrimitivePort {
			name: name,
			index: NullableIndex::none(),
			dir: PortDir::Input,
			node: NullableIndex::some(node),
			usr_idx: NullableIndex::some(usr_idx),
		}
	}
	pub fn output(name: IdString, node: StoreIndex<Node>) -> PrimitivePort {
		PrimitivePort {
			name: name,
			index: NullableIndex::none(),
			dir: PortDir::Output,
			node: NullableIndex::some(node),
			usr_idx: NullableIndex::none(),
		}
	}
	pub fn is_output(&self) -> bool {
		match self.dir {
			PortDir::Output => true,
			_ => false,
		}
	}
}

impl NamedItem for PrimitivePort {
	fn get_name(&self) -> IdString { self.name }
	fn set_name(&mut self, name: IdString) { self.name = name; }
	fn set_index(&mut self, index: StoreIndex<Self>) { self.index = NullableIndex::some(index); }
}

// An instance of a primitive
pub struct Primitive {
	pub name: IdString,
	pub index: NullableIndex<Self>,
	pub typ: PrimitiveType,
	pub attrs: FxHashMap<IdString, Constant>,
	pub ports: NamedStore<PrimitivePort>,
	pub src: SrcInfo,
}

impl Primitive {
	pub fn new(name: IdString, typ: PrimitiveType, src: SrcInfo) -> Primitive {
		Primitive {
			name: name,
			index: NullableIndex::none(),
			typ: typ,
			attrs: FxHashMap::default(),
			ports: NamedStore::new(),
			src: src,
		}
	}
}

impl NamedItem for Primitive {
	fn get_name(&self) -> IdString { self.name }
	fn set_name(&mut self, name: IdString) { self.name = name; }
	fn set_index(&mut self, index: StoreIndex<Self>) { self.index = NullableIndex::some(index); }
}