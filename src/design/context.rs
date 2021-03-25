use crate::StoreIndex;
use crate::core::{constids, BitVector, IdString, IdStringDb, NamedStore, OperandType};
use crate::design::{Node, PortRef, Primitive, PrimitivePort, PrimitiveType};

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
	pub fn add_node(&mut self, name: IdString, ty: OperandType, driver: StoreIndex<Primitive>, driver_port: IdString) -> Result<StoreIndex<Node>, String> {
		let node_idx = self.nodes.add(Node::new(name, ty, PortRef { prim: driver, port: driver_port }))?;
		self.prims.get_mut(driver).ports.add(PrimitivePort::output(name, node_idx))?;
		Ok(node_idx)
	}
	pub fn add_const(&mut self, ids: &mut IdStringDb, value: BitVector) -> StoreIndex<Node> {
		let value_ty = value.op_type();
		// Create a constant primitive
		let prim_name = self.auto_id(ids);
		let prim = self.add_prim(prim_name, PrimitiveType::Constant(value)).unwrap();
		// Create a constant node driven by it
		let node_name = self.auto_id(ids);
		self.add_node(node_name, value_ty, prim, constids::Q).unwrap()
	}
	pub fn remove_node(&mut self, node: StoreIndex<Node>) {
		// Before removing a node all users must be removed first
		assert_eq!(self.nodes.get(node).users.count(), 0);
		// Remove the output port of the associated primitive
		let drv = &self.nodes.get(node).driver;
		self.prims.get_mut(drv.prim).ports.remove_named(drv.port);
		// Remove the node itself
		self.nodes.remove(node);
	}
	pub fn trim_nodes(&mut self) -> usize {
		let dead_nodes : Vec<StoreIndex<Node>> = self.nodes.iter().filter_map(|(i, n)| if n.users.count() == 0 { Some(i) } else { None }).collect();
		for n in dead_nodes.iter() {
			self.nodes.remove(*n);
		}
		dead_nodes.len()
	}
	pub fn trim_prims(&mut self) -> usize {
		let dead_prims : Vec<StoreIndex<Primitive>> = self.prims.iter().filter_map(|(i, p)| if !p.ports.iter().any(|(_, port)| port.is_output()) { Some(i) } else { None }).collect();
		for p in dead_prims.iter() {
			self.prims.remove(*p);
		}
		dead_prims.len()
	}
	pub fn trim(&mut self) -> usize {
		let mut total_count = 0;
		loop {
			let iter_count = self.trim_nodes() + self.trim_prims();
			total_count += iter_count;
			// loop until nothing changes
			if iter_count == 0 { break; }
		}
		total_count
	}
}
