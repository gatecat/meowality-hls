use crate::core::IdString;
use crate::ast::base::*;
use crate::ast::{DataType, Expression, StructureDef, TemplateArg};

pub struct VariableDecl {
	pub name: IdString,
	pub ty: DataType,
	pub init: Option<Expression>,
	pub is_const: bool,
	pub is_static: bool,
}

pub struct TypedefDecl {
	pub name: IdString,
	pub ty: DataType,
}

pub struct UsingDecl {
	pub name: IdString,
	pub ty: DataType,
}

pub struct IfStatement {
	pub cond: Expression,
	pub if_true: Box<Statement>,
	pub if_false: Option<Box<Statement>>,
	pub is_meta: bool,
}

pub struct ForLoop {
	pub init: Box<Statement>,
	pub cond: Expression,
	pub incr: Expression,
	pub body: Box<Statement>,
	pub is_meta: bool,
}

pub struct MulticycleBlock {
	pub content: Box<Statement>,
}

pub struct FunctionArg {
	pub name: IdString,
	pub data_type: DataType,
	pub default: Option<Expression>,
	pub attrs: AttributeList,
}

pub struct Function {
	pub templ_args: Vec<TemplateArg>,
	pub func_args: Vec<FunctionArg>,
	pub ret_type: DataType,
	pub attrs: AttributeList,
	pub src: SrcInfo,
	pub content: Box<Statement>,
}

pub enum IODir {
	Input,
	Output,
	Interface,
}

pub struct ModuleIO {
	pub arg_type: DataType,
	pub name: IdString,
	pub dir: IODir,
}

pub struct ClockInfo {
	pub freq: f64,
	pub is_falling_edge: bool,
}

pub struct EnableInfo {}

pub struct ResetInfo {
	pub is_sync: bool,
	pub is_active_low: bool,
}

pub struct Module {
	pub templ_args: Vec<TemplateArg>,
	pub ports: Vec<ModuleIO>,
	pub clock: Option<ClockInfo>,
	pub enable: Option<EnableInfo>,
	pub reset: Option<ResetInfo>,
	pub attrs: AttributeList,
	pub src: SrcInfo,
	pub content: Box<Statement>,
}

pub enum StatementType {
	Null,
	Typedef(TypedefDecl),
	Using(UsingDecl),
	Var(VariableDecl),
	If(IfStatement),
	For(ForLoop),
	Block(Vec<Statement>),
	Multicycle(MulticycleBlock),
	Return(Expression),
	Break,
	Continue,
	Module(Module),
	Function(Function),
	Struct(StructureDef),
}

pub struct Statement {
	pub ty: StatementType,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}
