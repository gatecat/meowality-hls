use crate::core::{BitVector, IdString};
use crate::ast::base::*;
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
	Void,
	Auto,
	TemplParam(IdString),
	ScopedType(Box<DataType>, IdString),
	Integer(IntegerType),
	User(UserType),
	Reference(Box<DataType>),
	FIFO(FIFOType),
	Memory(MemoryType),
	Array(ArrayType),
}

pub enum TemplateArgType {
	Integer{t: IntegerType, default: Option<BitVector>},
	Typename{default: Option<DataType>},
}

pub struct TemplateArg {
	pub name: IdString,
	pub arg_type: TemplateArgType,
	pub attrs: AttributeList,
}

pub struct StructureDef {
	pub templ_args: Vec<TemplateArg>,
	pub items: Vec<Statement>,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}