
// Operand type
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct OperandType {
	pub width: usize,
	pub is_signed: bool,
}

impl OperandType {
	pub const fn new(width: usize, is_signed: bool) -> OperandType {
		OperandType { width, is_signed }
	}
	pub const fn signed(width: usize) -> OperandType {
		OperandType { width, is_signed: true }
	}
	pub const fn unsigned(width: usize) -> OperandType {
		OperandType { width, is_signed: true }
	}
	pub const fn extra_bit(a: OperandType, b: OperandType) -> usize {
		if a.is_signed != b.is_signed {1} else {0}
	}
	pub fn merge(a: OperandType, b: OperandType) -> OperandType {
		OperandType {
			width: std::cmp::max(a.width, b.width) + Self::extra_bit(a, b),
			is_signed: a.is_signed | b.is_signed,
		}
	}
	pub fn extend(self, add: usize) -> OperandType {
		OperandType {
			width: self.width + add, 
			is_signed: self.is_signed
		}
	}
	pub const BOOL : OperandType = OperandType::unsigned(1);
}

// The standard C type operations

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BasicOp {
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Neg,
	Eq,
	Neq,
	Gt,
	Lt,
	GtEq,
	LtEq,
	Shl,
	Shr,
	BwAnd,
	BwOr,
	BwXor,
	BwNot,
	LogAnd,
	LogOr,
	LogNot,
	LogCast,
}

impl BasicOp {
	pub fn result_width(&self, t: &[OperandType]) -> OperandType {
		use BasicOp::*;
		match &self {
			Add | Sub => OperandType::merge(t[0], t[1]).extend(1),
			Mul => OperandType::new(t[0].width + t[1].width + OperandType::extra_bit(t[0], t[1]), t[0].is_signed | t[1].is_signed),
			Div => OperandType::new(t[0].width + OperandType::extra_bit(t[0], t[1]), t[0].is_signed | t[1].is_signed),
			Mod => OperandType::new(t[1].width + OperandType::extra_bit(t[0], t[1]), t[0].is_signed | t[1].is_signed),
			Neg => OperandType::signed(t[0].width + 1),
			Eq | Neq | Gt | Lt | GtEq | LtEq => OperandType::BOOL,
			Shl | Shr => unimplemented!(), // icky case
			BwAnd | BwOr | BwXor => OperandType::merge(t[0], t[1]),
			BwNot => t[0],
			LogAnd | LogOr | LogNot | LogCast => OperandType::BOOL,
		}
	}
}