use crate::core::{Constant, IdString};
use crate::core::constids;
use std::fmt;

// A list of 'symbol' tokens in descending length order
const SYMBOLS : &[&'static str] = &[
	">>=",
	"<<=",
	"++",
	"--",
	">=",
	"<=",
	"+=",
	"-=",
	"*=",
	"/=",
	"%=",
	"&=",
	"|=",
	"^=",
	"&&",
	"||",
	"->",
	"==",
	"!=",
	">>",
	"<<",
	"{",
	"}",
	"[",
	"]",
	"<",
	">",
	"(",
	")",
	",",
	";",
	".",
	"+",
	"-",
	"*",
	"/",
	"%",
	"&",
	"^",
	"|",
	"=",
	"~",
	"!",
];

// A list of IdStrings that are _always_ parsed as keywords and not identifiers
const KEYWORDS : &[IdString] = &[
	constids::void,
	constids::int,
	constids::r#char,
	constids::r#string,
	constids::signed,
	constids::unsigned,
	constids::auto,
	constids::template,
	constids::namespace,
	constids::typedef,
	constids::using,
	constids::r#struct,
	constids::r#enum,
	constids::r#union,
	constids::r#if,
	constids::r#else,
	constids::r#for,
	constids::r#while,
	constids::multicycle,
	constids::meta,
	constids::r#break,
	constids::r#continue,
	constids::r#return,
	constids::r#sizeof,
	constids::block,
];

pub enum Token {
	Symbol(&'static str),
	Keyword(IdString),
	Ident(IdString),
	Literal(Constant),
}

impl fmt::Debug for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use Token::*;
		match self {
			Symbol(s) => write!(f, "token '{}'", s),
			Keyword(k) => write!(f, "keyword '{}'", k),
			Ident(i) => write!(f, "identifier '{}'", i),
			Literal(l) => write!(f, "literal '{:?}'", l),
		}
	}
}