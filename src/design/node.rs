use crate::ast::SrcInfo;
use crate::core::{IdString, ObjectStore, NullableIndex, StoreIndex, OperandType, NamedItem};
use crate::design::Primitive;

pub struct PortRef {
	pub prim: StoreIndex<Primitive>,
	pub port: IdString,
}

pub struct Node {
	pub name: IdString,
	pub index: NullableIndex<Self>,
	pub typ: OperandType,
	pub has_ready: bool,
	pub has_valid: bool,
	pub delay: Option<u64>,
	pub latency: Option<u32>,
	pub driver: PortRef,
	pub users: ObjectStore<PortRef>,
	pub src: SrcInfo,
}

impl Node {
	pub fn new(name: IdString, typ: OperandType, driver: PortRef, src: SrcInfo) -> Node {
		Node {
			name: name,
			index: NullableIndex::none(),
			typ: typ,
			has_ready: false, // TODO
			has_valid: false,
			delay: None,
			latency: None,
			driver: driver,
			users: ObjectStore::new(),
			src: src,
		}
	}
}

impl NamedItem for Node {
	fn get_name(&self) -> IdString { self.name }
	fn set_name(&mut self, name: IdString) { self.name = name; }
	fn set_index(&mut self, index: StoreIndex<Self>) { self.index = NullableIndex::some(index); }
}