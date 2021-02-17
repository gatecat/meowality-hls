use crate::ast::base::*;
use crate::ast::TemplateValue;
use crate::core::IdString;
use crate::core::BitVector;

macro_rules! operators {
	($(($tok:literal, $name:ident, $args:literal, $prec:literal, $postfix:literal, $right_assoc:literal)),*,) => {
		#[derive(Eq, PartialEq, Debug, Copy, Clone)]
		pub enum Operator {
			$($name),*
		}
		impl Operator {
			pub fn token(&self) -> &'static str {
				match self {
					$(Operator::$name => $tok),*
				}
			}
			pub fn arg_count(&self) -> u32 {
				match self {
					$(Operator::$name => $args),*
				}
			}
			pub fn precedence(&self) -> u32 {
				match self {
					$(Operator::$name => $prec),*
				}
			}
			pub fn is_postfix(&self) -> bool {
				match self {
					$(Operator::$name => $postfix),*
				}
			}
			pub fn is_right_assoc(&self) -> bool {
				match self {
					$(Operator::$name => $right_assoc),*
				}
			}
			pub const SYMBOLS : &'static [&'static str] = &[
				$($tok),*
			];
			pub fn lookup(sym: &'static str, arg_count: u32, postfix: bool) -> Option<Operator> {
				match (sym, arg_count, postfix) {
					$( ($tok, $args, $postfix) => Some(Operator::$name) ),*,
					(_, _, _) => None
				}
			}
		}
	};
}

operators! {
	("++",  PostInc,   2, 16, true,  false),
	("--",  PostDec,   2, 16, true,  false),
	("++",  PreInc,    2, 15, false, true),
	("--",  PreDec,    2, 15, false, true),
	("+",   Promote,   1, 15, false, true),
	("-",   Negate,    1, 15, false, true),
	("!",   LogNot,    1, 15, false, true),
	("~",   BitNot,    1, 15, false, true),
	("&",   RedAnd,    1, 15, false, true),
	("|",   RedOr,     1, 15, false, true),
	("^",   RedXor,    1, 15, false, true),
	("*",   Mul,       2, 13, false, false),
	("/",   Div,       2, 13, false, false),
	("%",   Mod,       2, 13, false, false),
	("+",   Add,       2, 12, false, false),
	("-",   Sub,       2, 12, false, false),
	("<<",  Shl,       2, 11, false, false),
	(">>",  Shr,       2, 11, false, false),
	("<",   Lt,        2,  9, false, false),
	(">",   Gt,        2,  9, false, false),
	("<=",  LtEq,      2,  9, false, false),
	(">=",  GtEq,      2,  9, false, false),
	("==",  Eq,        2,  8, false, false),
	("!=",  Neq,       2,  8, false, false),
	("&",   BwAnd,     2,  7, false, false),
	("^",   BwXor,     2,  6, false, false),
	("|",   BwOr,      2,  5, false, false),
	("&&",  LogAnd,    2,  4, false, false),
	("||",  LogOr,     2,  3, false, false),
	("=",   Assign,    2,  2, false, true),
	("+=",  AsAdd,     2,  2, false, true),
	("-=",  AsSub,     2,  2, false, true),
	("*=",  AsMul,     2,  2, false, true),
	("/=",  AsDiv,     2,  2, false, true),
	("%=",  AsMod,     2,  2, false, true),
	("<<=", AsShl,     2,  2, false, true),
	(">>=", AsShr,     2,  2, false, true),
	("&=",  AsAnd,     2,  2, false, true),
	("|=",  AsOr,      2,  2, false, true),
	("^=",  AsXor,     2,  2, false, true),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct FuncCall {
	pub target: Box<Expression>,
	pub targs: Vec<TemplateValue>,
	pub args: Vec<Expression>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ArrayAccess {
	pub array: Box<Expression>,
	pub indices: Vec<Expression>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct BitSlice {
	pub array: Box<Expression>,
	pub start: Box<Expression>,
	pub end: Box<Expression>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum BuiltinType {
	SizeOf,
	WidthOf,
	LengthOf,
	Pipeline,
	Delay,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ExprType {
	Null,
	Literal(BitVector),
	Variable(IdString),
	MemberAccess(Box<Expression>, IdString),
	TemplateArg(IdString),
	List(Vec<Expression>),
	Op(Operator, Vec<Expression>),
	Func(FuncCall),
	ArrAcc(ArrayAccess),
	Slice(BitSlice),
	Builtin(BuiltinType),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Expression {
	pub ty: ExprType,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}

impl Expression {
	pub fn new(ty: ExprType)  -> Expression {
		Expression {
			ty: ty,
			attrs: AttributeList::new(),
			src: SrcInfo::default(),
		}
	}
	pub fn new_full(ty: ExprType, attrs: AttributeList, src: SrcInfo) -> Expression {
		Expression {
			ty: ty,
			attrs: attrs,
			src: src,
		}
	}
	pub fn from_u64(i: u64, len: usize) -> Expression {
		Expression::new(ExprType::Literal(BitVector::from_u64(i, len)))
	}
	pub fn as_u64(&self) -> Option<u64> {
		match &self.ty {
			ExprType::Literal(x) => Some(x.as_u64()),
			_ => None
		}
	}
}
