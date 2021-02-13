use crate::core::{BitVector, IdString};
use crate::ast::base::*;
use crate::ast::{Expression, Statement};
use rustc_hash::FxHashMap;

pub struct IntegerType {
	pub width: Expression,
	pub is_signed: Expression,
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

pub enum DataTypes {
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

pub struct DataType {
	pub typ: DataTypes,
	pub is_static: bool,
	pub is_const: bool,
}

pub enum TemplateArgType {
	Value{t: DataType, default: Option<Expression>},
	Typename{default: Option<DataType>},
}

pub struct TemplateArg {
	pub name: IdString,
	pub arg_type: TemplateArgType,
	pub attrs: AttributeList,
}

impl TemplateArg {
	pub fn value(name: IdString, t: DataType, default: Option<Expression>, attrs: AttributeList) -> TemplateArg {
		TemplateArg {
			name: name,
			arg_type: TemplateArgType::Value {t: t, default: default},
			attrs: attrs,
		}
	}
	pub fn typename(name: IdString, default: Option<DataType>, attrs: AttributeList) -> TemplateArg {
		TemplateArg {
			name: name,
			arg_type: TemplateArgType::Typename {default: default},
			attrs: attrs,
		}
	}
}

pub struct StructureDef {
	pub name: IdString,
	pub templ_args: Vec<TemplateArg>,
	pub items: Vec<Statement>,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}