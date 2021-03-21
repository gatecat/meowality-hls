use std::ops::{BitAnd, BitOr, BitXor, Not};
use crate::core::{OperandType, StoreIndex};
use std::cmp::max;
use std::fmt;

// The four states that planar cares about
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum State {
	S0,
	S1,
	Sx, // undefined
	Sz, // high-impedance
}

impl State {
	pub fn to_val_mask(&self) -> (bool, bool)  {
		match &self {
			State::S0 => (false, false),
			State::S1 => (true, false),
			State::Sx => (false, true),
			State::Sz => (true, true),
		}
	}
	pub fn from_val_mask(val: bool, mask: bool) -> State {
		match (val, mask) {
			(false, false) => State::S0,
			(true, false) => State::S1,
			(false, true) => State::Sx,
			(true, true) => State::Sz,
		}
	}
	pub fn to_char(&self) -> char {
		match &self {
			State::S0 => '0',
			State::S1 => '1',
			State::Sx => 'x',
			State::Sz => 'z',
		}
	}
	pub fn from_char(c: char) -> Option<State> {
		match c {
			'0' => Some(State::S0),
			'1' => Some(State::S1),
			'x' => Some(State::Sx),
			'z' => Some(State::Sz),
			_ => None,
		}
	}
}

impl BitAnd for State {
	type Output = Self;
	fn bitand(self, r: State) -> State {
		use State::*;
		match (self, r) {
			(S1, S1) => S1,
			(S0, _) => S0,
			(_, S0) => S0,
			(_, _) => Sx
		}
	}
}

impl BitOr for State {
	type Output = Self;
	fn bitor(self, r: State) -> State {
		use State::*;
		match (self, r) {
			(S0, S0) => S0,
			(S1, _) => S1,
			(_, S1) => S1,
			(_, _) => Sx
		}
	}
}

impl BitXor for State {
	type Output = Self;
	fn bitxor(self, r: State) -> State {
		use State::*;
		match (self, r) {
			(S0, S0) => S0,
			(S1, S0) => S1,
			(S0, S1) => S1,
			(S1, S1) => S0,
			(_, _) => Sx
		}
	}
}

impl Not for State {
	type Output = Self;
	fn not(self) -> State {
		use State::*;
		match self {
			S0 => S1,
			S1 => S0,
			_ => Sx,
		}
	}
}

// An efficient way of storing four-value vectors;
// where we mainly care about the 'defined' part
//
// mask | val | state
//  0   |  0  |  S0
//  0   |  1  |  S1
//  1   |  0  |  Sx
//  1   |  1  |  Sz
#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct FourValChunk {
	pub value: u64,
	pub mask: u64,
}

impl FourValChunk {
	pub fn get(&self, idx: usize) -> State {
		assert!(idx < 64);
		State::from_val_mask(
			(self.value >> idx) & 0x1 == 0x1,
			(self.mask >> idx) & 0x1 == 0x1,
		)
	}
	pub fn set(&mut self, idx: usize, s: State) {
		assert!(idx < 64);
		let (val, mask) = s.to_val_mask();
		if val { self.value |= 1 << idx; } else { self.value &= !(1 << idx); }
		if mask { self.mask |= 1 << idx; } else { self.mask &= !(1 << idx); }
	}
}

// Our BitVector type is implemented as a list of the above chunks
// TODO: do we need to store signedness, too?
#[derive(Eq, PartialEq, Clone)]
pub struct BitVector {
	pub length: usize,
	pub is_signed: bool,
	pub chunks: Vec<FourValChunk>,
}

impl BitVector {
	pub fn new(len: usize, is_signed: bool) -> BitVector {
		BitVector {
			length: len,
			is_signed: is_signed,
			chunks: vec![ FourValChunk::default(); max(1, (len + 63) / 64)]
		}
	}
	pub fn from_u64(val: u64, len: usize) -> BitVector {
		let mut result = BitVector::new(len, false);
		if len > 0 {
			result.chunks[0].value = val;
		}
		return result;
	}
	pub fn from_i64(val: i64, len: usize) -> BitVector {
		let mut result = BitVector::new(len, true);
		if len > 0 {
			result.chunks[0].value = val as u64;
		}
		return result;
	}
	pub fn from_bits(bits: &[State]) -> BitVector {
		let mut result = BitVector::new(bits.len(), false);
		for (i, b) in bits.iter().enumerate() {
			result.chunks[i / 64].set(i % 64, *b);
		}
		return result;
	}
	pub fn len(&self) -> usize { self.length }
	pub fn get(&self, i: usize) -> Option<State> {
		if i < self.length {
			Some(self.chunks[i / 64].get(i % 64))
		} else {
			None
		}
	}
	pub fn get_ext(&self, i: usize) -> State {
		if i < self.length {
			self.chunks[i / 64].get(i % 64)
		} else {
			if self.length == 0 || !self.is_signed {
				State::S0
			} else {
				self.get_ext(self.length - 1)
			}
		}
	}
	pub fn set(&mut self, i: usize, s: State) {
		self.chunks[i / 64].set(i % 64, s)
	}
	pub fn is_defined(&self) -> bool {
		self.chunks.iter().all(|c| c.mask == 0)
	}
	pub fn has_undef(&self) -> bool {
		self.chunks.iter().any(|c| c.mask != 0)
	}
	pub fn as_u64(&self) -> u64 {
		// converts any undef to 0
		self.chunks[0].value & (!self.chunks[0].mask)
	}
	pub fn as_def_u64(&self) -> Option<u64> {
		// converts any undef to None
		if self.chunks[0].mask == 0 {
			Some(self.chunks[0].value)
		} else {
			None
		}
	}
	pub fn to_str(&self) -> String {
		(0..self.len()).rev().map(|i| self.get(i).unwrap().to_char()).collect()
	}
	pub fn from_str(s: &str) -> BitVector {
		let mut result = BitVector::new(s.len(), false);
		for (i, c) in s.chars().rev().enumerate() {
			result.chunks[i / 64].set(i % 64, State::from_char(c).unwrap());
		}
		return result;
	}
	pub fn op_type(&self) -> OperandType {
		OperandType::new(self.length, self.is_signed)
	}
	pub fn iter(&self) -> impl Iterator<Item = State> + '_ {
		(0..self.len()).map(move |i| self.get(i).unwrap())
	}
}

impl fmt::Display for BitVector {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.to_str())
	}
}
impl fmt::Debug for BitVector {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}{}", if self.is_signed { "s" } else { "" }, self.to_str())
	}
}

// This stores the available choices for an enumerated parameter
#[derive(Eq, PartialEq, Clone)]
pub struct EnumType {
	pub type_name: String,
	pub choices: Vec<String>,
}

// A constant can be
//  - a bitvector (this is the most common case)
//  - a string (used for various parameter types)
//  - an enumerated value (generally parsed from a string)
#[derive(Eq, PartialEq, Clone)]
pub enum Constant {
	Bits(BitVector),
	Str(String),
	Enum(StoreIndex<EnumType>, u32),
}

impl Constant {
	pub fn from_yosys_str(s: &str) -> Constant {
		match s.find(|c| c != '0' && c != '1' && c != 'x' && c != 'z') {
			// All chars match 01xz - it's a bitvector
			None => Constant::Bits(BitVector::from_str(s)),
			Some(i) => {
				// It's a string
				// If all chars of the string would be 01xz, disambiguated with an extra space at the end
				match s[i..].find(|c| c != ' ') {
					None => Constant::Str(s[0..s.len()-1].to_string()),
					Some(_) => Constant::Str(s.to_string())
				}
			}
		}
	}
}

impl fmt::Debug for Constant {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use Constant::*;
		match &self {
			Bits(b) => write!(f, "{}'b{}", b.len(), b.to_str()),
			Str(s) => write!(f, "\"{}\"", s),
			Enum(t, v) => write!(f, "enum({}, {})", t.index(), v),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn int_vec() {
		let v0 = BitVector::from_u64(69, 16);
		assert_eq!(v0.len(), 16);
		assert_eq!(v0.as_u64(), 69);
		assert_eq!(v0.as_def_u64(), Some(69));
		assert!(v0.is_defined());

		let v1 = BitVector::from_u64(0xFEDCBA9876543210, 64);
		assert_eq!(v1.as_u64(), 0xFEDCBA9876543210);
	}

	#[test]
	fn get_bits() {
		use State::*;
		let bits : &[State] = &[S0, S1, Sx, Sz, Sx, Sz, S1, S1];
		let v = BitVector::from_bits(bits);
		assert_eq!(v.len(), 8);
		assert_eq!(v.get(0), Some(S0));
		assert_eq!(v.get(1), Some(S1));
		assert_eq!(v.get(2), Some(Sx));
		assert_eq!(v.get(3), Some(Sz));
		assert_eq!(v.get(5), Some(Sz));
		assert_eq!(v.get(7), Some(S1));
		assert_eq!(v.get(8), None);
		assert_eq!(v.as_u64(), 0b11000010);
		assert!(!v.is_defined());
		assert!(v.has_undef());
	}

	#[test]
	fn wide() {
		use State::*;
		let mut bits : Vec<State> = Vec::new();
		for i in 0..4096 {
			bits.push(match i % 3 {
				0 => S0,
				1 => Sx,
				_ => S1,
			})
		}
		let v = BitVector::from_bits(&bits);
		assert_eq!(v.len(), 4096);
		assert_eq!(v.get(0), Some(S0));
		assert_eq!(v.get(1), Some(Sx));
		assert_eq!(v.get(2), Some(S1));
		assert_eq!(v.get(2047), Some(Sx));
		assert_eq!(v.get(3000), Some(S0));
		assert_eq!(v.get(4094), Some(S1));
		assert_eq!(v.get(4095), Some(S0));
		assert_eq!(v.get(4096), None);
	}

	#[test]
	fn to_string() {
		use State::*;
		let bits : &[State] = &[S0, S1, Sx, Sz, Sx, Sz, S1, S1];
		let v = BitVector::from_bits(bits);
		assert_eq!(v.to_string(), "11zxzx10");
	}

	#[test]
	fn from_yosys_str() {
		use State::*;
		assert_eq!(Constant::from_yosys_str("101"), Constant::Bits(BitVector::from_u64(5, 3)));
		assert_eq!(Constant::from_yosys_str("10xz1"), Constant::Bits(BitVector::from_bits(&[S1, Sz, Sx, S0, S1])));
		assert_eq!(Constant::from_yosys_str("10xz1 "), Constant::Str("10xz1".to_string()));
		assert_eq!(Constant::from_yosys_str("10xz1  "), Constant::Str("10xz1 ".to_string()));
		assert_eq!(Constant::from_yosys_str("foo"), Constant::Str("foo".to_string()));
	}

	#[test]
	fn debug_const() {
		assert_eq!(format!("{:?}", Constant::from_yosys_str("100")), "3'b100");
		assert_eq!(format!("{:?}", Constant::from_yosys_str("10xz1")), "5'b10xz1");
		assert_eq!(format!("{:?}", Constant::from_yosys_str("10xz1 ")), "\"10xz1\"");
	}
}
