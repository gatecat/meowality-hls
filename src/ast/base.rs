use crate::core::IdString;
use crate::ast::Expression;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct LineCol {
	pub line: u32,
	pub col: u32,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SrcInfo {
	pub file: IdString,
	pub start: LineCol,
}

impl SrcInfo {
	pub fn default() -> SrcInfo {
		SrcInfo {
			file: IdString::NONE,
			start: LineCol { line: 0, col: 0},
		}
	}
}

pub struct Attribute {
	pub name: IdString,
	pub value: Option<Expression>,
}

pub struct AttributeList(Vec<Attribute>);

impl AttributeList {
	pub fn new() -> AttributeList {
		AttributeList(Vec::new())
	}
}