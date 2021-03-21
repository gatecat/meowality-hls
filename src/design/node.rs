use crate::core::{IdString, ObjectStore, StoreIndex, OperandType, NamedItem};

pub enum PortRef {
	TopLevel(IdString), // external I/O port
	Prim(IdString, IdString), // primitive, with port name
}

pub struct Node {
	pub name: IdString,
	pub index: StoreIndex<Self>,
	pub ty: OperandType,
	pub has_ready: bool,
	pub has_valid: bool,
	pub is_input: bool,
	pub is_output: bool,
	pub delay: Option<u64>,
	pub latency: Option<u32>,
	pub driver: PortRef,
	pub users: ObjectStore<PortRef>,
}

impl NamedItem for Node {
	fn get_name(&self) -> IdString { self.name }
	fn set_name(&mut self, name: IdString) { self.name = name; }
	fn set_index(&mut self, index: StoreIndex<Self>) { self.index = index; }
}