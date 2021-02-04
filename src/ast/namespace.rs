use crate::core::IdString;
use crate::ast::base::*;
use crate::ast::Statement;

pub enum NamespaceItem {
	Namespace(Box<Namespace>),
	Stmt(Statement),
}

pub struct Namespace {
	pub name: Option<IdString>,
	pub content: Option<NamespaceItem>,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}