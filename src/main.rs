#![allow(dead_code)]
#![feature(deque_range)]
#![feature(min_const_generics)]

pub mod ast;
pub mod core;
pub mod design;
pub mod parser;

use std::io::Read;
use std::fs::File;
use crate::core::*;
use crate::parser::*;

use std::env;

fn main() -> Result<(), String> {
	let args: Vec<String> = env::args().collect();
	let mut ids = IdStringDb::new();
	constids::do_ids_init(&mut ids);
	let filename = args.get(1).unwrap();
	let mut f = File::open(filename).map_err(|e| e.to_string())?;
	let mut buffer = String::new();
	f.read_to_string(&mut buffer).map_err(|e| e.to_string())?;
	let tokeniser = Tokeniser::new(ids.id(filename), buffer.chars());
	let ps = ParserState::new(tokeniser, &mut ids).map_err(|e| e.msg.to_string())?;
	let mut p = Parser::new(ps);
	let sts = p.do_parse(&mut ids).map_err(|e| e.msg.to_string())?;
	for st in sts.iter() {
		println!("{}", st);
	}
	Ok(())
}
