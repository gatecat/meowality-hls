use crate::core::IdString;
use crate::ast::{Expression, Statement};
use rustc_hash::FxHashMap;

pub struct IntegerType {
	pub width: usize,
	pub is_signed: bool,
}

pub struct UserType {
	pub name: IdString,
	pub args: FxHashMap<IdString, Expression>,
}

pub struct FIFOType {
	pub base: Box<DataType>,
	pub depth: Expression,
}

pub struct MemoryType {
	pub base: Box<DataType>,
	pub depth: Expression,
}

pub struct ArrayType {
	pub base: Box<DataType>,
	pub dims: Vec<Expression>,
}

pub enum DataType {
	Null,
	Auto,
	Integer(IntegerType),
	User(UserType),
	FIFO(FIFOType),
	Memory(MemoryType),
	Array(ArrayType),
}

pub enum TemplateType {
	Integer(IntegerType),
	Typename,
}

pub struct StructureDefinition {
	pub templ_args: Vec<TemplateType>,
	pub items: Vec<Statement>,
}