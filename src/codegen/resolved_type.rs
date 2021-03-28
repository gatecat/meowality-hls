use crate::core::{BitVector, OperandType};
use crate::codegen::Identifier;

// Resolved template arguments
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ResolvedArg {
	Const(BitVector),
	Type(ResolvedType),
}

// Key for a function or structure with resolved template arguments
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ResolvedKey {
	pub name: Identifier,
	pub templ_args: Vec<ResolvedArg>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ResolvedType {
	Void,
	Integer(OperandType),
	AutoInt,
	Reference(Box<ResolvedType>),
	Array(Box<ResolvedType>, usize),
	Struct(ResolvedKey),
}
