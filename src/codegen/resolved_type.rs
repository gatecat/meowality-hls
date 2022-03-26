use crate::ast::{DataType, Function};
use rustc_hash::FxHashMap;
use crate::core::IdString;
use crate::core::{BitVector, OperandType};
use crate::codegen::{CodegenError, Identifier};
use crate::codegen::eval::Eval;
use std::fmt;

// Resolved template arguments
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ResolvedArg {
	Const(BitVector),
	Type(ResolvedType),
}

impl fmt::Debug for ResolvedArg {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		match &self {
			ResolvedArg::Const(c) => write!(fmt, "{}", c)?,
			ResolvedArg::Type(t) => write!(fmt, "{:?}", t)?,
		}
		Ok(())
	}
}

// Key for a function or structure with resolved template arguments
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ResolvedKey {
	pub name: Identifier,
	pub templ_args: Vec<ResolvedArg>,
}

impl fmt::Debug for ResolvedKey {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(fmt, "{:?}", self.name)?;
		if !self.templ_args.is_empty() {
			write!(fmt, "<")?;
			for arg in self.templ_args.iter() { write!(fmt, "{:?}, ", arg)?; }
			write!(fmt, ">")?;
		}
		Ok(())
	}
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ResolvedTypes {
	Void,
	Integer(OperandType),
	AutoInt,
	Reference(Box<ResolvedType>),
	Array(Box<ResolvedType>, usize),
	Struct(ResolvedKey),
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ResolvedType {
	pub typ: ResolvedTypes,
	pub is_const: bool,
	pub is_static: bool,
}

impl fmt::Debug for ResolvedType {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		use ResolvedTypes::*;
		if self.is_static { write!(fmt, "static ")? };
		if self.is_const { write!(fmt, "const ")? };
		match &self.typ {
			Void => write!(fmt, "void")?,
			Integer(ot) => write!(fmt, "{:?}", ot)?,
			AutoInt => write!(fmt, "auto_int")?,
			Reference(t) => write!(fmt, "{:?}&", t)?,
			Array(a, l) => write!(fmt, "{:?}[{}]", a, l)?,
			Struct(k) => write!(fmt, "{:?}", k)?,
		}
		Ok(())
	}
}

impl ResolvedType {
	pub fn merge(&self, other: &ResolvedType) -> Option<ResolvedType> {
		use ResolvedTypes::*;
		Some(ResolvedType {
			is_const: (self.is_const | other.is_const),
			is_static: (self.is_static | other.is_static),
			typ: match &self.typ {
				Void => Some(other.typ.clone()),
				Integer(ot1) => match other.typ {
					Integer(ot2) => Some(Integer(OperandType::merge(*ot1, ot2))),
					AutoInt => Some(AutoInt),
					Void => Some(Integer(*ot1)),
					_ => None,
				}
				AutoInt => match other.typ {
					Integer(_) | AutoInt | Void => Some(AutoInt),
					_ => None,
				},
				Reference(rt1) => match &other.typ {
					Reference(rt2) => Some(Reference(Box::new(rt1.merge(&rt2)?))),
					_ => Some(Reference(Box::new(rt1.merge(&other)?))),
				},
				Array(base, len) => match &other.typ {
					Void => Some(Array(base.clone(), *len)),
					Array(base2, len2) => Some(Array(Box::new(base.merge(base2)?), std::cmp::max(*len, *len2))),
					_ => None,
				},
				Struct(k) => {
					match &other.typ {
						Void => Some(Struct(k.clone())),
						Struct(k2) => if k == k2 { Some(Struct(k.clone())) } else { None },
						_ => None,
					}
				},
			}?
		})
	}
	pub fn do_resolve<'a>(e: &'a mut Eval<'a>, dt: &DataType) -> Result<ResolvedType, CodegenError> {
		use crate::ast::DataTypes;
		use ResolvedTypes::*;
		let base_type = match &dt.typ {
			DataTypes::Void => ResolvedType {typ: Void, is_static: false, is_const: false},
			DataTypes::Integer(i) => {
				let width = e.const_eval_scalar(&i.width)?.as_u64() as usize;
				let is_signed = e.const_eval_scalar(&i.is_signed)?.as_u64() != 0;
				ResolvedType {typ: Integer(OperandType::new(width, is_signed)), is_static: false, is_const: false}
			},
			_ => unimplemented!()
		};
		Ok(base_type)
	}
}

// A derived structure
pub struct DerivedStruct {
	pub members: FxHashMap<IdString, ResolvedType>,
	pub functions: Vec<Function>, 
}

impl DerivedStruct {
	pub fn new() -> DerivedStruct {
		DerivedStruct {
			members: FxHashMap::default(),
			functions: Vec::new(),
		}
	}
}