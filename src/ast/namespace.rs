use crate::core::IdString;
use crate::ast::base::*;
use crate::ast::{Statement};

pub enum NamespaceItem {
	Namespace(Box<Namespace>),
	Stmt(Statement),
}

pub struct Namespace {
	pub name: Option<IdString>,
	pub content: Vec<NamespaceItem>,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}

impl Namespace {
	pub fn new(name: Option<IdString>, attrs: AttributeList, src: SrcInfo) -> Namespace {
		Namespace {
			name: name,
			content: Vec::new(),
			attrs: attrs,
			src: src,
		}
	}
}
