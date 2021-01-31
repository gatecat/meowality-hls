use crate::core::IdString;
pub struct NodeType {
	pub is_signed: bool,
	pub width: usize,
}

pub struct Node {
	pub name: IdString,
	pub ty: NodeType,
	pub has_ready: bool,
	pub has_valid: bool,
	pub is_input: bool,
	pub is_output: bool,
	pub delay: Option<u64>,
	pub latency: Option<u32>,
}