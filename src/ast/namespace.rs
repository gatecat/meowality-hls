use crate::core::IdString;
use crate::ast::base::*;
use crate::ast::{Scope, Statement};

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

impl Scope for Namespace {
	fn is_type(&self, ident: IdString) -> bool {
		self.content.iter().any(|c| match c { NamespaceItem::Stmt(s) => s.leaf_is_type(ident), _ => false })
	}
	fn is_func(&self, ident: IdString) -> bool {
		self.content.iter().any(|c| match c { NamespaceItem::Stmt(s) => s.leaf_is_func(ident), _ => false })
	}
	fn is_var(&self, ident: IdString) -> bool {
		self.content.iter().any(|c| match c { NamespaceItem::Stmt(s) => s.leaf_is_func(ident), _ => false })
	}
	fn get_decls<'a>(&'a self) -> Vec<&'a Statement> {
		self.content.iter().filter_map(|c| match c { NamespaceItem::Stmt(s) => Some(s), _ => None }).collect()
	}
}