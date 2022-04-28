use rustc_hash::FxHashMap;
use crate::core::{IdString, BitVector, BasicOp, OperandType, Constant};
use crate::design::PortDir;

/*
Our final IL, as a shim between a Design and a backend

This lowers away many of the complex primitives used in a Design; as well as things like un-abstracting clocks
and ready/valid handshaking.

The end result is something that is more-or-less directly serialisable to RTLIL
*/
pub enum LowPrim {
	Null,
	Input,
	Op {ty: BasicOp, a: IdString, b: IdString},
	Const {val: BitVector},
	Assign {src: IdString},
	BitSelect {src: IdString, start: usize, end: usize},
	Cat {src: Vec<IdString>},
	Mux {src: Vec<IdString>, sel: IdString},
}

pub struct LowNode {
	pub name: IdString,
	pub dir: Option<PortDir>,
	pub value: LowPrim,
	pub attrs: FxHashMap<IdString, Constant>,
	pub typ: OperandType,
}

pub struct LowNetlist {
	pub name: IdString,
	pub nodes: FxHashMap<IdString, LowNode>,
}
