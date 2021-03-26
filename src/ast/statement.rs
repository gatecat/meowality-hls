use std::fmt;

use crate::core::IdString;
use crate::ast::base::*;
use crate::ast::{DataType, Expression, StructureDef, TemplateArg};

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
	pub fn leaf_is_var(&self, ident: IdString) -> bool {
		match &self.ty {
			Var(s) => s.name == ident,
			Func(s) => s.name == ident,
			_ => false,
		}
	}
	pub fn children<'a>(&'a self) -> StatementIter<'a> {
		StatementIter {
			st: &self,
			i: 0,
		}
	}

	fn write_targs<T: std::fmt::Write>(stream: &mut T, args: &[TemplateArg]) -> fmt::Result {
		if !args.is_empty() {
			write!(stream, "template <")?;
			for arg in args.iter() {
				write!(stream, "{}, ", arg)?;
			}
			write!(stream, "> ")?;
		}
		Ok(())
	}

	pub fn dump<T: std::fmt::Write>(&self, stream: &mut T, indent: usize, newline: bool) -> fmt::Result {
		write!(stream, "{:indent$}", "", indent=indent)?;
		use StatementType::*;
		match &self.ty {
			Null => write!(stream, ";")?,
			Typedef(td) => write!(stream, "typedef {} {:?};", td.ty, td.name)?,
			Using(ud) => write!(stream, "using {} = {:?};", ud.ty, ud.name)?,
			Var(v) => {
				write!(stream, "{} {:?}", v.ty, v.name)?;
				if let Some(i) = &v.init { write!(stream, " = {}", i)?; };
				write!(stream, ";")?;
			},
			If(i) => {
				writeln!(stream, "if {}({})", if i.is_meta {"meta "} else {""}, i.cond)?;
				i.if_true.dump(stream, indent + 2, true)?;
				if let Some(f) = &i.if_false {
					writeln!(stream, "{:indent$}else", "", indent=indent)?;
					f.dump(stream, indent + 2, true)?;
				}
			},
			For(f) => {
				writeln!(stream, "for {}(", if f.is_meta {"meta "} else {""})?;
				f.init.dump(stream, 0, false)?;
				writeln!(stream, "{};{})", f.cond, f.incr)?;
				f.body.dump(stream, indent + 2, true)?;
			},
			Block(b) => {
				writeln!(stream, "{{")?;
				for s in b.iter() { s.dump(stream, indent + 2, true)?; }
				writeln!(stream, "{:indent$}}}", "", indent=indent)?;
			},
			Return(e) => write!(stream, "return {};", e)?,
			Break => write!(stream, "break;")?,
			Continue => write!(stream, "continue;")?,
			Module(m) => {
				Self::write_targs(stream, &m.templ_args)?;
				write!(stream, "module {:?}(", m.name)?;
				if let Some(_) = &m.clock { write!(stream, "clock, ")?; }
				if let Some(_) = &m.enable { write!(stream, "enable, ")?; }
				if let Some(_) = &m.reset { write!(stream, "reset, ")?; }
				for p in m.ports.iter().filter(|p| p.dir == IODir::Input) {
					write!(stream, "{} {:?},", p.arg_type, p.name)?;
				}
				write!(stream, ") -> (")?;
				for p in m.ports.iter().filter(|p| p.dir == IODir::Output) {
					write!(stream, "{} {:?},", p.arg_type, p.name)?;
				}
				writeln!(stream, ")")?;
				m.content.dump(stream, indent + 2, true)?;
			},
			Func(f) => {
				Self::write_targs(stream, &f.templ_args)?;
				write!(stream, "{} {:?}(", f.ret_type, f.name)?;
				for a in f.func_args.iter() {
					write!(stream, "{} {:?}", a.data_type, a.name)?;
					if let Some(d) = &a.default { write!(stream, " = {}", d)?; }
					write!(stream, ",")?;
				}
				write!(stream, ")")?;
				f.content.dump(stream, indent + 2, true)?;
			},
			Struct(s) => {
				Self::write_targs(stream, &s.templ_args)?;
				write!(stream, "struct {:?}", s.name)?;
				s.block.dump(stream, indent + 2, true)?;
			},
			Expr(e) => write!(stream, "{}", e)?,
			_ => unimplemented!(),
		}
		if newline { writeln!(stream, "")?; }
		Ok(())
	}
}

impl fmt::Display for Statement {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.dump(f, 0, true)?;
		Ok(())
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
