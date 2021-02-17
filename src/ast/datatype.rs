use crate::core::{BitVector, IdString};
use crate::ast::base::*;
use crate::ast::{Expression, Statement};

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct IntegerType {
	pub width: Expression,
	pub is_signed: Expression,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum TemplateValue {
	Expr(Expression),
	Typ(DataType),
}

impl TemplateValue {
	pub fn as_expr(&self) -> Option<Expression> {
		match self {
			TemplateValue::Expr(e) => Some(e.clone()),
			_ => None,
		}
	}
	pub fn as_type(&self) -> Option<DataType> {
		match self {
			TemplateValue::Typ(t) => Some(t.clone()),
			_ => None,
		}
	}
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct UserType {
	pub name: IdString,
	pub args: Vec<TemplateValue>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct FIFOType {
	pub base: Box<DataType>,
	pub depth: Expression,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct MemoryType {
	pub base: Box<DataType>,
	pub depth: Expression,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ArrayType {
	pub base: Box<DataType>,
	pub dims: Vec<Expression>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum DataTypes {
	Void,
	Auto,
	AutoInt,
	TemplParam(IdString),
	ScopedType(Box<DataType>, IdString),
	Integer(IntegerType),
	User(UserType),
	Reference(Box<DataType>),
	FIFO(FIFOType),
	Memory(MemoryType),
	Array(ArrayType),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct DataType {
	pub typ: DataTypes,
	pub is_static: bool,
	pub is_const: bool,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum TemplateArgType {
	Value{t: DataType, default: Option<Expression>},
	Typename{default: Option<DataType>},
}

#[derive(Eq, PartialEq, Debug, Clone)]
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

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct StructureDef {
	pub name: IdString,
	pub templ_args: Vec<TemplateArg>,
	pub block: Box<Statement>,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}