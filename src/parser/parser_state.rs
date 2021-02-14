use std::collections::VecDeque;

use crate::ast::LineCol;
use crate::core::{IdString, IdStringDb};
use crate::parser::{Token, Tokeniser, TokeniserError};
use Token::*;

pub struct ParserState<Iter: Iterator<Item=char>>  {
	tokeniser: Tokeniser<Iter>,
	toks: VecDeque<(Token, LineCol)>,
	ptr: usize,
	ambig_stack: Vec<usize>,
}

#[derive(Eq, PartialEq, Debug)]
pub struct ParserError {
	pub file: IdString,
	pub lc: LineCol,
	pub msg: String,
}

impl ParserError {
	pub fn from_tok(e: TokeniserError) -> ParserError {
		ParserError {
			file: e.file,
			lc: e.lc,
			msg: e.msg,
		}
	}
}

impl <Iter: Iterator<Item=char>> ParserState<Iter> {
	pub fn new(tok: Tokeniser<Iter>, ids: &mut IdStringDb) -> Result<ParserState<Iter>, ParserError> {
		let mut p = ParserState {
			tokeniser: tok,
			toks: VecDeque::new(),
			ptr: 0,
			ambig_stack: Vec::new(),
		};
		p.update_lookahead(ids, 1)?;
		Ok(p)
	}
	pub fn update_lookahead(&mut self, ids: &mut IdStringDb, n: usize) -> Result<(), ParserError> {
		while self.toks.len() < (self.ptr + n) && !self.tokeniser.eof() {
			let next_tok = self.tokeniser.token(ids).or_else(|e| Err(ParserError::from_tok(e)))?;
			self.toks.push_back((next_tok, self.tokeniser.linecol()));
		}
		Ok(())
	}
	pub fn peek(&self) -> Option<&(Token, LineCol)> {
		self.toks.get(self.ptr)
	}
	pub fn err(&self, msg: String) -> ParserError {
		ParserError {
			file: self.tokeniser.file(),
			lc: self.tokeniser.linecol(),
			msg: msg,
		}
	}
	pub fn get(&mut self, ids: &mut IdStringDb) -> Result<(Token, LineCol), ParserError> {
		self.update_lookahead(ids, 2)?;
		if !self.ambig_stack.is_empty() {
			let t = self.toks.get(self.ptr).ok_or(self.err(format!("unexpected end of file")))?;
			self.ptr += 1;
			Ok(t.clone())
		} else {
			self.toks.pop_front().ok_or(self.err(format!("unexpected end of file")))
		}
	}
	// In ambiguous mode, we don't really eat the tokens but just pretend to; as we might need to backtrack
	pub fn enter_ambig(&mut self) {
		self.ambig_stack.push(self.ptr)
	}
	// Ambiguous mode ended successfully, found an unambiguous match, now commit by actually eating the tokens
	pub fn ambig_success(&mut self, ids: &mut IdStringDb) -> Result<(), ParserError> {
		let next_ptr = self.ambig_stack.pop().unwrap();
		for _ in next_ptr..self.ptr {
			self.toks.pop_front();
		}
		self.ptr = next_ptr;
		self.update_lookahead(ids, 1)?;
		Ok(())
	}
	// Ambiguous mode hasn't resolved (yet), backtrack so we can try parsing differently
	pub fn ambig_failure(&mut self, ids: &mut IdStringDb) -> Result<(), ParserError> {
		let next_ptr = self.ambig_stack.pop().unwrap();
		self.ptr = next_ptr;
		self.update_lookahead(ids, 1)?;
		Ok(())
	}
	// Check if a symbol is the next token
	pub fn check_sym(&self, sym: &'static str) -> bool {
		match self.peek() {
			Some((Symbol(s), _)) => (*s == sym),
			_ => false,
		}
	}
	// Check if a symbol is one of a given list of keywords
	pub fn check_kws(&self, kws: &[IdString]) -> bool {
		match self.peek() {
			Some((Keyword(s), _)) => kws.iter().any(|kw| s == kw),
			Some((Ident(s), _)) => kws.iter().any(|kw| s == kw),
			_ => false,
		}
	}
	// Check if a symbol is the next token; and consume it if it is
	pub fn consume_sym(&mut self, ids: &mut IdStringDb, sym: &'static str) -> Result<bool, ParserError> {
		if self.check_sym(sym) {
			self.get(ids)?;
			Ok(true)
		} else {
			Ok(false)
		}
	}
	// Check if a keyword is the next token; and consume it if it is
	pub fn consume_kw(&mut self, ids: &mut IdStringDb, kw: IdString) -> Result<bool, ParserError> {
		match self.peek() {
			Some((Keyword(x), _)) | Some((Ident(x), _)) => {
				if *x == kw {
					self.get(ids)?; // consume
					Ok(true)
				} else {
					Ok(false)
				}
			}
			_ => Ok(false)
		}
	}
	// Check if an identifier is the next token; and consume it if it is
	pub fn consume_ident(&mut self, ids: &mut IdStringDb) -> Result<Option<IdString>, ParserError> {
		match self.peek() {
			Some((Ident(x), _)) => { let x = *x; self.get(ids)?; Ok(Some(x)) },
			_ => Ok(None)
		}
	}
	// Check if an literal is the next token; and consume it if it is
	pub fn consume_literal(&mut self, ids: &mut IdStringDb) -> Result<Option<Token>, ParserError> {
		let next = self.peek();
		match next {
			Some((IntLiteral(_), _)) | Some((ChrLiteral(_), _)) | Some((StrLiteral(_), _)) 
				=> { let next = next.unwrap().0.clone(); self.get(ids)?; Ok(Some(next)) },
			_ => Ok(None)
		}
	}
	// Expect a symbol as the next token
	pub fn expect_sym(&mut self, ids: &mut IdStringDb, sym: &'static str) -> Result<(), ParserError>  {
		if !self.consume_sym(ids, sym)? {
			Err(self.err(format!("expected '{}', got {:?}", sym, self.peek().unwrap().0)))
		} else {
			Ok(())
		}
	}
	// Expect a keyword as the next token
	pub fn expect_kw(&mut self, ids: &mut IdStringDb, kw: IdString) -> Result<(), ParserError>  {
		if !self.consume_kw(ids, kw)? {
			Err(self.err(format!("expected '{}', got {:?}", kw, self.peek().unwrap().0)))
		} else {
			Ok(())
		}
	}
	// Expect an identifier as the next token
	pub fn expect_ident(&mut self, ids: &mut IdStringDb) -> Result<IdString, ParserError>  {
		match self.consume_ident(ids)? {
			Some(x) => Ok(x),
			_ => Err(self.err(format!("expected identifier, got {:?}", self.peek().unwrap().0)))
		}
	}
	pub fn expect_literal(&mut self, ids: &mut IdStringDb) -> Result<Token, ParserError>  {
		match self.consume_literal(ids)? {
			Some(x) => Ok(x),
			_ => Err(self.err(format!("expected literal, got {:?}", self.peek().unwrap().0)))
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::core::constids;
	#[test]
	fn symbols() -> Result<(), ParserError> {
		let mut ids = IdStringDb::new();
		constids::do_ids_init(&mut ids);
		let tok = Tokeniser::new(ids.id("<test>"), "{++!=".chars());
		let mut ps = ParserState::new(tok, &mut ids)?;
		assert_eq!(ps.consume_sym(&mut ids, "++")?, false);
		assert_eq!(ps.consume_sym(&mut ids, "{")?, true);
		assert_eq!(ps.expect_sym(&mut ids, "++"), Ok(()));
		assert_eq!(ps.expect_sym(&mut ids, "--").unwrap_err().msg, "expected '--', got token '!='");
		Ok(())
	}
	#[test]
	fn ident_keyword() -> Result<(), ParserError> {
		let mut ids = IdStringDb::new();
		constids::do_ids_init(&mut ids);
		let tok = Tokeniser::new(ids.id("<test>"), "struct foo{".chars());
		let mut ps = ParserState::new(tok, &mut ids)?;
		assert_eq!(ps.consume_kw(&mut ids, constids::r#if)?, false);
		assert_eq!(ps.consume_kw(&mut ids, constids::r#struct)?, true);
		assert_eq!(ps.consume_ident(&mut ids)?, Some(ids.id("foo")));
		assert_eq!(ps.expect_ident(&mut ids).unwrap_err().msg, "expected identifier, got token '{'");
		Ok(())
	}
	#[test]
	fn ambig_mode() -> Result<(), ParserError> {
		let mut ids = IdStringDb::new();
		constids::do_ids_init(&mut ids);
		let tok = Tokeniser::new(ids.id("<test>"), "{ mytype::foo bar".chars());
		let mut ps = ParserState::new(tok, &mut ids)?;
		assert_eq!(ps.consume_sym(&mut ids, "{")?, true);
		ps.enter_ambig();
		assert_eq!(ps.consume_ident(&mut ids)?, Some(ids.id("mytype")));
		assert_eq!(ps.consume_sym(&mut ids, "::")?, true);
		assert_eq!(ps.consume_ident(&mut ids)?, Some(ids.id("foo")));
		ps.ambig_failure(&mut ids)?;
		ps.enter_ambig();
		assert_eq!(ps.consume_ident(&mut ids)?, Some(ids.id("mytype")));
		assert_eq!(ps.consume_sym(&mut ids, "::")?, true);
		assert_eq!(ps.consume_ident(&mut ids)?, Some(ids.id("foo")));
		ps.ambig_success(&mut ids)?;
		assert_eq!(ps.consume_ident(&mut ids)?, Some(ids.id("bar")));
		Ok(())
	}
}