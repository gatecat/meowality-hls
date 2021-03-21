use crate::core::{State, BitVector};

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
		OperandType { width, is_signed: false }
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
	pub fn result_type(&self, t: &[OperandType]) -> OperandType {
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
	pub fn apply(&self, operands: &[BitVector]) -> BitVector {
		use BasicOp::*;
		// Up to two arguments
		let mut types = [OperandType::BOOL; 2];
		for (i, op) in operands.iter().enumerate() {
			types[i] = op.op_type();
		}
		let result_type = self.result_type(&types[0..operands.len()]);
		let mut result = BitVector::new(result_type.width, result_type.is_signed);
		match &self {
			Add => {
				let mut carry = State::S0;
				for i in 0..result.len() {
					let a = operands[0].get_ext(i);
					let b = operands[1].get_ext(i);
					result.set(i, a ^ b ^ carry);
					carry = (a & b) | (a & carry) | (b & carry);	
				}
			},
			BwAnd => for i in 0..result.len() { result.set(i, operands[0].get_ext(i) & operands[1].get_ext(i)); },
			BwOr => for i in 0..result.len() { result.set(i, operands[0].get_ext(i) | operands[1].get_ext(i)); },
			BwXor => for i in 0..result.len() { result.set(i, operands[0].get_ext(i) ^ operands[1].get_ext(i)); },
			BwNot => for i in 0..result.len() { result.set(i, !operands[0].get_ext(i)); },
			LogAnd => result = BwAnd.apply(&[LogCast.apply(&[operands[0].clone()]), LogCast.apply(&[operands[1].clone()])]),
			LogOr => result = BwOr.apply(&[LogCast.apply(&[operands[0].clone()]), LogCast.apply(&[operands[1].clone()])]),
			LogNot => result = BwNot.apply(&[LogCast.apply(&[operands[0].clone()])]),
			LogCast => {
				result.set(0, if operands[0].iter().any(|b| b == State::S1) { State::S1 } else { State::S0 });
			},
			_ => unimplemented!()
		}
		result
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn add() {
		assert_eq!(BasicOp::Add.apply(&[BitVector::from_u64(42, 8), BitVector::from_u64(69, 8)]), BitVector::from_u64(111, 9));
		assert_eq!(BasicOp::Add.apply(&[BitVector::from_u64(42, 8), BitVector::from_i64(-1, 8)]), BitVector::from_i64(41, 10));
	}
	#[test]
	fn bitwise() {
		assert_eq!(BasicOp::BwAnd.apply(&[BitVector::from_u64(0b11000110, 8), BitVector::from_u64(0b101, 3)]), BitVector::from_u64(0b100, 8));
		assert_eq!(BasicOp::BwOr.apply(&[BitVector::from_u64(0b11000110, 8), BitVector::from_u64(0b101, 3)]), BitVector::from_u64(0b11000111, 8));
		assert_eq!(BasicOp::BwXor.apply(&[BitVector::from_u64(0b11000110, 8), BitVector::from_u64(0b101, 3)]), BitVector::from_u64(0b11000011, 8));
		assert_eq!(BasicOp::BwNot.apply(&[BitVector::from_u64(0b11000110, 8)]), BitVector::from_u64(0b00111001, 8));
	}
	#[test]
	fn logical() {
		assert_eq!(BasicOp::LogCast.apply(&[BitVector::from_u64(0b0000, 4)]), BitVector::from_u64(0b0, 1));
		assert_eq!(BasicOp::LogCast.apply(&[BitVector::from_u64(0b1010, 4)]), BitVector::from_u64(0b1, 1));
		assert_eq!(BasicOp::LogOr.apply(&[BitVector::from_u64(0b1010, 4), BitVector::from_u64(0b0, 1)]), BitVector::from_u64(0b1, 1));
		assert_eq!(BasicOp::LogAnd.apply(&[BitVector::from_u64(0b1010, 4), BitVector::from_u64(0b0, 1)]), BitVector::from_u64(0b0, 1));
		assert_eq!(BasicOp::LogAnd.apply(&[BitVector::from_u64(0b1010, 4), BitVector::from_u64(0b1, 1)]), BitVector::from_u64(0b1, 1));
		assert_eq!(BasicOp::LogNot.apply(&[BitVector::from_u64(0b1010, 4)]), BitVector::from_u64(0b0, 1));
	}
}