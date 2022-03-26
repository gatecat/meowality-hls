#![allow(dead_code)]
#![feature(deque_range)]
#![feature(min_const_generics)]

pub mod ast;
pub mod core;
pub mod design;
pub mod parser;
pub mod codegen;

use std::io::Read;
use std::fs::File;
use crate::core::*;
use crate::parser::*;
use crate::ast::*;
use crate::codegen::*;

use std::env;

fn conv_ids(ids: &IdStringDb, s: &str) -> String {
	let mut result = String::new();
	let mut buf = String::new();
	let mut in_id = false;
	for c in s.chars() {
		if !in_id {
			// not in IdString
			if c == '`' {
				in_id = true;
			} else {
				result.push(c);
			}
		} else {
			if c == '`' {
				result.push_str(ids.get_str(IdString { index: buf.parse::<u32>().unwrap() }));
				buf.clear();
				in_id = false;
			} else {
				buf.push(c);
			}
		}
	}
	result
}

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
	println!("*** RAW AST ***");
	for st in sts.iter() {
		let raw_st = format!("{}", st);
		println!("{}", &conv_ids(&ids, &raw_st));
	}
	for st in sts.iter() {
		if let StatementType::Module(m) = &st.ty {
			println!("*** MODULE {} ***", ids.get_str(m.name));
			let mut e = crate::codegen::eval::Eval::init(&mut ids, m.name);
			e.eval_st(&m.content).map_err(|e| e.1.to_string())?;
			for (_, v) in e.st.vars.iter() {
				println!("  {:?}: {:?} = {:?}", ids.get_str(v.name), v.typ, v.value);
			}
		}
	}
	Ok(())
}
