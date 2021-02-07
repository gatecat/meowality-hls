use std::collections::VecDeque;

use crate::ast::LineCol;
use crate::core::{BitVector, IdString, IdStringDb};
use crate::parser::{Token, Tokeniser, TokeniserError};
use Token::*;

struct Parser<Iter: Iterator<Item=char>>  {
	tokeniser: Tokeniser<Iter>,
	toks: VecDeque<(Token, LineCol)>,
	ptr: usize,
	ambig_mode: bool,
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

impl <Iter: Iterator<Item=char>> Parser<Iter> {
	pub fn new(tok: Tokeniser<Iter>, ids: &mut IdStringDb) -> Result<Parser<Iter>, ParserError> {
		let mut p = Parser {
			tokeniser: tok,
			toks: VecDeque::new(),
			ptr: 0,
			ambig_mode: false,
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
		if self.ambig_mode {
			let t = self.toks.get(self.ptr).ok_or(self.err(format!("unexpected end of file")))?;
			self.ptr += 1;
			Ok(t.clone())
		} else {
			self.toks.pop_front().ok_or(self.err(format!("unexpected end of file")))
		}
	}
	pub fn enter_ambig(&mut self) {
		// In ambiguous mode, we don't really eat the tokens but just pretend to; as we might need to backtrack
		self.ptr = 0;
		self.ambig_mode = true;
	}
	pub fn ambig_success(&mut self, ids: &mut IdStringDb) -> Result<(), ParserError> {
		// Ambiguous mode ended successfully, found an unambiguous match, now commit by actually eating the tokens
		for _ in 0..self.ptr {
			self.toks.pop_front();
		}
		self.ptr = 0;
		self.ambig_mode = false;
		self.update_lookahead(ids, 1)?;
		Ok(())
	}
	pub fn ambig_failure(&mut self, ids: &mut IdStringDb) -> Result<(), ParserError> {
		// Ambiguous mode hasn't resolved (yet), backtrack so we can try parsing differently
		self.ptr = 0;
		self.ambig_mode = false;
		self.update_lookahead(ids, 1)?;
		Ok(())
	}
}
