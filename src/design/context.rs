use crate::StoreIndex;
use crate::core::{IdString, IdStringDb, NamedStore};
use crate::design::{Node, Primitive, PrimitiveType};

pub struct Context {
	pub ids: IdStringDb,
}

pub struct Design {
	pub name: IdString,
	pub nodes: NamedStore<Node>,
	pub prims: NamedStore<Primitive>,
	pub top_ports: Vec<StoreIndex<Primitive>>,
	auto_idx: usize,
}

impl Design {
	pub fn new(name: IdString) -> Design {
		Design {
			name: name,
			nodes: NamedStore::new(),
			prims: NamedStore::new(),
			top_ports: Vec::new(),
			auto_idx: 0,
		}
	}
	pub fn auto_id(&mut self, ids: &mut IdStringDb) -> IdString {
		let id = ids.id(&format!("$auto${}", self.auto_idx));
		self.auto_idx += 1;
		id
	}
	pub fn add_prim(&mut self, name: IdString, ty: PrimitiveType) -> Result<StoreIndex<Primitive>, String> {
		self.prims.add(Primitive::new(name, ty))
	}
}