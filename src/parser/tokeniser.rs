use std::collections::VecDeque;
use crate::parser::token::*;
use crate::ast::LineCol;
use crate::core::{Constant, BitVector, IdString, IdStringDb};

// Fast matching of ASCII characters
pub struct CharPool {
	chars: u128,
}

impl CharPool {
	pub fn from_iter<Iter: Iterator<Item=char>>(mut iter: Iter)  -> CharPool {
		let mut p = CharPool {
			chars: 0,
		};
		let mut buf : [u8; 1] = [0];
		loop {
			match iter.next() {
				Some(c) => {
					c.encode_utf8(&mut buf);
					assert!(buf[0] < 128);
					p.chars |= 1 << (buf[0] as u128);
				}
				None => break
			}
		}
		return p;
	}
	pub fn is_match(&self, ch: char) -> bool {
		if ch.len_utf8() > 1 {
			return false;
		}
		let mut buf : [u8; 1] = [0];
		ch.encode_utf8(&mut buf);
		if buf[0] >= 128 {
			return false;
		}
		return ((self.chars >> buf[0]) & 0x1) == 0x1;
	}
}

pub struct Tokeniser<Iter: Iterator<Item=char>> {
	iter: Iter,
	filename: IdString,
	lookahead: VecDeque<char>,
	lineno: u32,
	colno: u32,
	// some internal helper structures
	max_symbol_len: usize,
	first_symbol_chars: CharPool,
	buf: String,
}

pub struct TokeniserError {
	pub file: IdString,
	pub lc: LineCol,
	pub msg: String,
}

impl <Iter: Iterator<Item=char>> Tokeniser<Iter> {
	pub fn new(filename: IdString, iter: Iter) -> Tokeniser<Iter> {
		let mut state = Tokeniser {
			iter: iter,
			filename: filename,
			lookahead: VecDeque::new(),
			lineno: 1,
			colno: 1,
			max_symbol_len: SYMBOLS.iter().map(|x| x.len()).max().unwrap_or(0),
			first_symbol_chars: CharPool::from_iter(SYMBOLS.iter().map(|x| x.chars().nth(0).unwrap())),
			buf: String::new(),
		};
		state.update_lookahead(1);
		return state;
	}
	pub fn update_lookahead(&mut self, n: usize) {
		while self.lookahead.len() < n {
			match self.iter.next() {
				Some(x) => self.lookahead.push_back(x),
				None => break,
			}
		}
	}
	pub fn eof(&self) -> bool {
		self.lookahead.is_empty()
	}
	pub fn peek(&self) -> Option<char> {
		self.lookahead.get(0).cloned()
	}
	pub fn get(&mut self) -> Option<char> {
		self.update_lookahead(2);
		let c = self.lookahead.pop_front();
		self.colno += 1;
		if c == Some('\n') {
			self.lineno += 1;
			self.colno = 1;
		}
		return c;
	}
	pub fn assert_get(&mut self, ch: char) {
		let next = self.get();
		assert_eq!(next, Some(ch));
	}
	pub fn skip_whitespace(&mut self) {
		while self.peek().map(|x| x.is_whitespace()).unwrap_or(false) {
			self.get();
		}
	}
	pub fn file(&self) -> IdString {
		self.filename
	}
	pub fn linecol(&self) -> LineCol {
		LineCol {
			line: self.lineno,
			col: self.colno,
		}
	}
	pub fn err(&self, s: String) -> TokeniserError {
		TokeniserError {
			file: self.file(),
			lc: self.linecol(),
			msg: s,
		}
	}
	pub fn is_symbol_start(&self, ch: char) -> bool {
		self.first_symbol_chars.is_match(ch)
	}
	pub fn get_string(&mut self, str_type: char) -> Result<(), TokeniserError> {
		let mut escaped = false;
		self.buf.clear();
		self.assert_get(str_type);
		loop {
			let ch = self.get();
			if ch.is_none() {
				return Err(self.err(format!("EOF in string literal")));
			}
			let ch = ch.unwrap();
			if escaped {
				match ch {
					'a' => self.buf.push('\x07'),
					'b' => self.buf.push('\x08'),
					'e' => self.buf.push('\x1B'),
					'f' => self.buf.push('\x0C'),
					'n' => self.buf.push('\n'),
					'r' => self.buf.push('\r'),
					't' => self.buf.push('\t'),
					'v' => self.buf.push('\x0B'),
					'\\' => self.buf.push('\\'),
					'\'' => self.buf.push('\''),
					'\"' => self.buf.push('\"'),
					'?' => self.buf.push('?'),
					'0' => self.buf.push('\0'),
					// TODO: hex and octal sequences
					c => { return Err(self.err(format!("unexpected {} in escape sequence", c))); }
				}
			} else {
				if ch == str_type {
					break;
				} else if ch == '\\' {
					escaped = true;
				} else {
					self.buf.push(ch);
				}
			}
		}
		Ok(())
	}
	pub fn token(&mut self, ids: &mut IdStringDb) -> Result<Token, TokeniserError> {
		self.skip_whitespace();
		let ch0 = self.peek().ok_or(self.err(format!("end of file")))?;
		if self.is_symbol_start(ch0) {
			// parse as symbol token
			self.update_lookahead(self.max_symbol_len);
			for s in SYMBOLS {
				if s.chars().enumerate().all(|(i, c)| *self.lookahead.get(i).unwrap() == c) {
					return Ok(Token::Symbol(s));
				}
			}
			return Err(self.err(format!("unexpected symbol {}", ch0)))
		} else if ch0.is_ascii_digit() {
			// parse as numeric literal
			self.buf.clear();
			while !self.eof() {
				let ch = self.peek().unwrap();
				if ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'' {
					// _ and ' are used as digit separators 
					if ch != '_' && ch != '\'' {
						self.buf.push(ch);
					}
					self.get();
				} else {
					break;
				}
			}
			// TODO: actual parsing
		} else if ch0 == '\"' {
			// parse as string literal
		} else if ch0 == '\'' {
			// parse as char (or multi-char) literal
		} else {
			// parse as identifier
			self.buf.clear();
			while !self.eof() {
				let ch = self.peek().unwrap();
				if !ch.is_whitespace() && !self.is_symbol_start(ch) {
					self.buf.push(ch);
					self.get();
				} else {
					break;
				}
			}
			let id = ids.id(&self.buf);
			if KEYWORDS.iter().any(|kw| *kw == id) {
				return Ok(Token::Keyword(id));
			} else {
				return Ok(Token::Ident(id));
			}
		}
		Err(self.err(format!("unexpected token starting with {}", ch0)))
	}
}