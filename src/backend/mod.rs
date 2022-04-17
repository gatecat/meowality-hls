use crate::core::{IdString, BitVector, BasicOp, OperandType, Constant};
use crate::design::PortDir;
use rustc_hash::FxHashMap;

pub trait Backend {
	fn init(&mut self, mod_name: IdString);
	fn add_node(&mut self, name: IdString, typ: OperandType, attrs: &FxHashMap<IdString, Constant>);
	fn add_port(&mut self, name: IdString, typ: OperandType, dir: PortDir, attrs: &FxHashMap<IdString, Constant>);
	fn add_const(&mut self, val: BitVector) -> IdString;
	fn add_basicop(&mut self, name: IdString, typ: BasicOp, inputs: &[IdString], output: IdString, attrs: &FxHashMap<IdString, Constant>);
	fn add_mux(&mut self, name: IdString, a: IdString, b: IdString, s: IdString, q: IdString, attrs: &FxHashMap<IdString, Constant>);
	fn add_dff(&mut self, name: IdString, clk: IdString, rst: IdString, en: IdString, d: IdString, q: IdString, attrs: &FxHashMap<IdString, Constant>);
	fn add_assign(&mut self, src: IdString, dst: IdString);
	fn finalise(&mut self);
}
