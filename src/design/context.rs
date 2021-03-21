use crate::core::{IdString, IdStringDb, NamedStore};
use crate::design::{Node, Primitive};

pub struct Context {
	pub ids: IdStringDb,
}

pub struct Design {
	pub name: IdString,
	pub node: NamedStore<Node>,
	pub prims: NamedStore<Primitive>,
}