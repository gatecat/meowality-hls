use crate::core::IdString;
use crate::ast::{DataType, Expression};

pub struct VariableDecl {
	pub name: IdString,
	pub ty: DataType,
	pub init: Option<Expression>,
	pub is_const: bool,
	pub is_static: bool,
}

pub struct IfStatement {
	pub cond: Expression,
	pub if_true: Box<Statement>,
	pub if_false: Option<Box<Statement>>,
}

pub struct ForLoop {
	pub init: Box<Statement>,
	pub cond: Expression,
	pub incr: Expression,
	pub body: Box<Statement>,
}

pub enum StatementType {
	Null,
	Var(VariableDecl),
	If(IfStatement),
	For(ForLoop),
	Block(Vec<Statement>),
	Return(Expression),
	Break,
	Continue,
}

pub struct Statement {
	pub ty: StatementType,
}