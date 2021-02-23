use crate::core::IdString;
use crate::ast::base::*;
use crate::ast::{DataType, Expression, StructureDef, TemplateArg, TemplateArgType};

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct VariableDecl {
	pub name: IdString,
	pub ty: DataType,
	pub init: Option<Expression>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TypedefDecl {
	pub name: IdString,
	pub ty: DataType,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct UsingDecl {
	pub name: IdString,
	pub ty: DataType,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct IfStatement {
	pub cond: Expression,
	pub if_true: Box<Statement>,
	pub if_false: Option<Box<Statement>>,
	pub is_meta: bool,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ForLoop {
	pub init: Box<Statement>,
	pub cond: Expression,
	pub incr: Expression,
	pub body: Box<Statement>,
	pub is_meta: bool,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct MulticycleBlock {
	pub content: Box<Statement>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct FunctionArg {
	pub name: IdString,
	pub data_type: DataType,
	pub default: Option<Expression>,
	pub attrs: AttributeList,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Function {
	pub name: IdString,
	pub templ_args: Vec<TemplateArg>,
	pub func_args: Vec<FunctionArg>,
	pub ret_type: DataType,
	pub attrs: AttributeList,
	pub src: SrcInfo,
	pub content: Box<Statement>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum IODir {
	Input,
	Output,
	Interface,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ModuleIO {
	pub arg_type: DataType,
	pub name: IdString,
	pub dir: IODir,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ClockInfo {
	pub freq: u64,
	pub is_falling_edge: bool,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct EnableInfo {}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ResetInfo {
	pub is_sync: bool,
	pub is_active_low: bool,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Module {
	pub name: IdString,
	pub templ_args: Vec<TemplateArg>,
	pub ports: Vec<ModuleIO>,
	pub clock: Option<ClockInfo>,
	pub enable: Option<EnableInfo>,
	pub reset: Option<ResetInfo>,
	pub attrs: AttributeList,
	pub src: SrcInfo,
	pub content: Box<Statement>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
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
	Func(Function),
	Struct(StructureDef),
	Expr(Expression),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Statement {
	pub ty: StatementType,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}

use StatementType::*;

impl Statement {
	pub fn new(ty: StatementType, attrs: AttributeList) -> Statement {
		Statement {
			ty: ty,
			attrs: attrs,
			src: SrcInfo::default(),
		}
	}
	pub fn num_children(&self) -> usize {
		match &self.ty {
			If(i) => if i.if_false.is_some() { 2 } else { 1 },
			For(_) => 2,
			Block(s) => s.len(),
			Multicycle(_) => 1,
			Func(_) => 1,
			Module(_) => 1,
			Struct(_) => 1,
			_ => 0,
		}
	}
	pub fn child(&self, i: usize) -> &Statement {
		match &self.ty {
			If(s) => match i {
				0 => { return &s.if_true },
				1 => { return s.if_false.as_ref().unwrap() },
				_ => {}
			}
			For(s) => match i {
				0 => { return &s.init },
				1 => { return &s.body },
				_ => {}
			}
			Block(s) => { return s.get(i).unwrap() },
			Multicycle(s) => if i == 0 { return &s.content },
			Module(s) => if i == 0 { return &s.content },
			Func(s) => if i == 0 { return &s.content },
			Struct(s) => if i == 0 { return &s.block },
			_ => {}
		}
		panic!("invalid statement child request");
	}
	pub fn templ_args(&self) -> &[TemplateArg] {
		match &self.ty {
			Func(s) => { return &s.templ_args },
			Struct(s) => { return &s.templ_args },
			Module(s) => { return &s.templ_args },
			_ => &[],
		}
	}
	// ** Non-recursive ** versions of the Scope functions
	pub fn leaf_is_type(&self, ident: IdString) -> bool {
		match &self.ty {
			Struct(s) => s.name == ident,
			Using(s) => s.name == ident,
			Typedef(s) => s.name == ident,
			_ => false,
		}
	}
	pub fn leaf_is_func(&self, ident: IdString) -> bool {
		match &self.ty {
			Func(s) => s.name == ident,
			_ => false,
		}
	}
	pub fn leaf_is_var(&self, ident: IdString) -> bool {
		match &self.ty {
			Var(s) => s.name == ident,
			_ => false,
		}
	}
	pub fn children<'a>(&'a self) -> StatementIter<'a> {
		StatementIter {
			st: &self,
			i: 0,
		}
	}
}

pub struct StatementIter<'a> {
	st: &'a Statement,
	i: usize,
}

impl <'a> Iterator for StatementIter<'a> {
	type Item = &'a Statement;
	fn next(&mut self) -> Option<Self::Item> {
		if self.i >= self.st.num_children() {
			None
		} else {
			let result = self.st.child(self.i);
			self.i += 1;
			Some(result)
		}
	}
}
