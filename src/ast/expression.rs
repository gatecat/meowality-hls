use crate::ast::base::*;
use crate::core::IdString;
use crate::core::BitVector;

macro_rules! operators {
	($(($tok:literal, $name:ident, $args:literal, $prec:literal, $postfix:literal)),*,) => {
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
		}
	};
}

operators! {
	("++",  PostInc,   2, 16, true),
	("--",  PostDec,   2, 16, true),
	("++",  PreInc,    2, 15, false),
	("--",  PreDec,    2, 15, false),
	("+",   Promote,   1, 15, false),
	("-",   Negate,    1, 15, false),
	("!",   LogNot,    1, 15, false),
	("~",   BitNot,    1, 15, false),
	("&",   RedAnd,    1, 15, false),
	("|",   RedOr,     1, 15, false),
	("^",   RedXor,    1, 15, false),
	("*",   Mul,       2, 13, false),
	("/",   Div,       2, 13, false),
	("%",   Mod,       2, 13, false),
	("+",   Add,       2, 12, false),
	("-",   Sub,       2, 12, false),
	("<<",  Shl,       2, 11, false),
	(">>",  Shr,       2, 11, false),
	("<",   Lt,        2,  9, false),
	(">",   Gt,        2,  9, false),
	("<=",  LtEq,      2,  9, false),
	(">=",  GtEq,      2,  9, false),
	("==",  Eq,        2,  8, false),
	("!=",  Neq,       2,  8, false),
	("&",   BwAnd,     2,  7, false),
	("^",   BwXor,     2,  6, false),
	("|",   BwOr,      2,  5, false),
	("&&",  LogAnd,    2,  4, false),
	("||",  LogOr,     2,  3, false),
	("=",   Assign,    2,  2, false),
	("+=",  AsAdd,     2,  2, false),
	("-=",  AsSub,     2,  2, false),
	("*=",  AsMul,     2,  2, false),
	("/=",  AsDiv,     2,  2, false),
	("%=",  AsMod,     2,  2, false),
	("<<=", AsShl,     2,  2, false),
	(">>=", AsShr,     2,  2, false),
	("&=",  AsAnd,     2,  2, false),
	("|=",  AsOr,      2,  2, false),
	("^=",  AsXor,     2,  2, false),
}

pub struct FuncCall {
	pub func_name: IdString,
	pub func_dest: Option<Box<Expression>>, // if a member function; this is what it is being called on
	pub func_args: Vec<Expression>,
}

pub struct ArrayAccess {
	pub array: Box<Expression>,
	pub indices: Vec<Expression>,
}

pub struct BitSlice {
	pub array: Box<Expression>,
	pub start: Box<Expression>,
	pub end: Box<Expression>,
}

pub enum BuiltinType {
	SizeOf,
	WidthOf,
	LengthOf,
	Pipeline,
	Delay,
}

pub enum ExprType {
	Null,
	Literal(BitVector),
	Variable(IdString),
	ScopedVariable(Box<Expression>, IdString),
	TemplateArg(IdString),
	List(Vec<Expression>),
	Op(Operator),
	Func(FuncCall),
	ArrAcc(ArrayAccess),
	Slice(BitSlice),
	Builtin(BuiltinType),
}

pub struct Expression {
	pub ty: ExprType,
	pub attrs: AttributeList,
	pub src: SrcInfo,
}

impl Expression {
	pub fn new(ty: ExprType, attrs: AttributeList, src: SrcInfo) -> Expression {
		Expression {
			ty: ty,
			attrs: attrs,
			src: src,
		}
	}
}