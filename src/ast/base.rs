use crate::core::IdString;
use crate::ast::Expression;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct LineCol {
	pub line: u32,
	pub col: u32,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Attribute {
	pub name: IdString,
	pub value: Expression,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct AttributeList(pub Vec<Attribute>);

impl AttributeList {
	pub fn new() -> AttributeList {
		AttributeList(Vec::new())
	}
}