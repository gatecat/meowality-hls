use crate::ast::Function;
use rustc_hash::FxHashMap;
use crate::core::IdString;
use crate::core::{BitVector, OperandType};
use crate::codegen::Identifier;
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