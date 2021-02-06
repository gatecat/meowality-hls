use crate::core::IdString;
use crate::ast::Expression;

pub struct LineCol {
	pub line: u32,
	pub col: u32,
}

pub struct SrcInfo {
	pub file: IdString,
	pub start: LineCol,
	pub end: LineCol,
}

pub struct Attribute {
	pub name: IdString,
	pub value: Option<Expression>,
}

pub struct AttributeList(Vec<Attribute>);
