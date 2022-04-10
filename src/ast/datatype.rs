use std::fmt;
use crate::core::{IdString};
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

impl fmt::Display for DataType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use DataTypes::*;
		if self.is_static { write!(f, "static ")?; }
		if self.is_const { write!(f, "const ")?; }
		match &self.typ {
			Void => write!(f, "void")?,
			Auto => write!(f, "auto")?,
			AutoInt => write!(f, "auto_int")?,
			TemplParam(t) => write!(f, "{:?}", t)?,
			ScopedType(base, t) => write!(f, "{}::{:?}", base, t)?,
			Integer(it) => write!(f, "integer<{}, {}>", it.is_signed, it.width )?,
			User(ut) => {
				write!(f, "{}", ut.name)?;
				if !ut.args.is_empty() {
					write!(f, "<")?;
					for a in ut.args.iter() { write!(f, "{:?}", a)?; }
					write!(f, ">")?;
				}
			},
			Reference(r) => write!(f, "{}&", r)?,
			Array(a) => {
				write!(f, "{}[", a.base)?;
				// TODO: template
				for e in a.dims.iter() { write!(f, "{},", e)?; }
				write!(f, "]")?;
			},
			_ => unimplemented!()
		}
		Ok(())
	}
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

impl fmt::Display for TemplateArg {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.arg_type {
			TemplateArgType::Value{t, default} => {
				write!(f, "{} {}", t, self.name)?;
				if let Some(d) = default { write!(f, " = {}", d)?; }
			},
			TemplateArgType::Typename{default} => {
				write!(f, "typename {}", self.name)?;
				if let Some(d) = default { write!(f, " = {}", d)?; }
			},
		}
		Ok(())
	}
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct StructureDef {
	pub name: IdString,
	pub is_interface: bool,
	pub templ_args: Vec<TemplateArg>,
	pub block: Box<Statement>,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}