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
		self.prims.get_mut(driver).ports.add(PrimitivePort::output(driver_port, node_idx))?;
		Ok(node_idx)
	}
	pub fn add_prim_input(&mut self, prim: StoreIndex<Primitive>, name: IdString, node: StoreIndex<Node>) -> Result<StoreIndex<PrimitivePort>, String> {
		let usr_idx = self.nodes.get_mut(node).users.add(PortRef { prim: prim, port: name });
		self.prims.get_mut(prim).ports.add(PrimitivePort::input(name, node, usr_idx))
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
	pub fn disconnect_port(&mut self, prim: StoreIndex<Primitive>, port: IdString) {
		{
			let p = self.prims.get(prim).ports.named(port).unwrap();
			self.nodes.get_mut(p.node.unwrap()).users.remove(p.usr_idx.unwrap());
		}
		self.prims.get_mut(prim).ports.remove_named(port);
	}
	pub fn remove_prim(&mut self, prim: StoreIndex<Primitive>) {
		// Before removing a primitive all outputs and associated nodes must be removed first
		assert_eq!(self.prims.get(prim).ports.iter().filter(|(_, p)| p.is_output()).count(), 0);
		// Then disconnect any inputs
		let inputs: Vec<IdString> = self.prims.get(prim).ports.iter().map(|(_, p)| p.name).collect();
		for ip in inputs.iter() {
			self.disconnect_port(prim, *ip);
		}
		// Finally remove the primitive
		self.prims.remove(prim);
	}
	pub fn trim_nodes(&mut self) -> usize {
		let dead_nodes : Vec<StoreIndex<Node>> = self.nodes.iter().filter_map(|(i, n)| if n.users.count() == 0 { Some(i) } else { None }).collect();
		for n in dead_nodes.iter() {
			self.remove_node(*n);
		}
		dead_nodes.len()
	}
	pub fn trim_prims(&mut self) -> usize {
		let dead_prims : Vec<StoreIndex<Primitive>> = self.prims.iter().filter_map(|(i, p)| if !p.ports.iter().any(|(_, port)| port.is_output()) { Some(i) } else { None }).collect();
		for p in dead_prims.iter() {
			self.remove_prim(*p);
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

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn const_prim() -> Result<(), String>
	{
		let mut ids = IdStringDb::new();
		constids::do_ids_init(&mut ids);
		let mut des = Design::new(ids.id("top"));
		let const_node = des.add_const(&mut ids, BitVector::from_u64(0xDEADBEEF, 32));
		assert_eq!(des.nodes.get(const_node).typ, OperandType::unsigned(32));
		assert_eq!(des.prims.get(des.nodes.get(const_node).driver.prim).typ, PrimitiveType::Constant(BitVector::from_u64(0xDEADBEEF, 32)));
		Ok(())
	}
	#[test]
	fn trim() -> Result<(), String> {
		let mut ids = IdStringDb::new();
		constids::do_ids_init(&mut ids);
		let mut des = Design::new(ids.id("top"));
		des.add_const(&mut ids, BitVector::from_u64(0xDEADBEEF, 32));
		assert_eq!(des.nodes.count(), 1);
		assert_eq!(des.prims.count(), 1);
		assert_eq!(des.trim(), 2);
		assert_eq!(des.nodes.count(), 0);
		assert_eq!(des.prims.count(), 0);
		Ok(())
	}
}