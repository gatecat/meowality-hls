use rustc_hash::FxHashMap;
use crate::ast::*;
use crate::core::{constids, IdString, IdStringDb};
use crate::parser::parser_state::*;
use crate::parser::token::*;

struct Parser<Iter: Iterator<Item=char>> {
	state: ParserState<Iter>,
	// for scope resolution
	namespace_stack: Vec<Namespace>,
	statement_stack: Vec<Statement>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum OpStackItem {
	Op(Operator),
	LParen,
	RParen,
}

const INTEGRAL_TYPES: &[IdString] = &[
	constids::signed,
	constids::unsigned,
	constids::char,
	constids::short,
	constids::long,
	constids::int,
];

impl <Iter: Iterator<Item=char>> Parser<Iter> {
	pub fn new(state: ParserState<Iter>) -> Parser<Iter> {
		Parser {
			state: state,
			namespace_stack: Vec::new(),
			statement_stack: Vec::new(),
		}
	}
	pub fn lookup_ident(&self, curr_scope: &dyn Scope, ident: IdString) -> Option<IdentifierType> {
		let curr_lookup = curr_scope.lookup_ident(ident);
		if curr_lookup.is_some() {
			return curr_lookup;
		}
		for st in self.statement_stack.iter().rev() {
			let st_lookup = st.lookup_ident(ident);
			if st_lookup.is_some() {
				return st_lookup;
			}
		}
		for ns in self.namespace_stack.iter().rev() {
			let ns_lookup = ns.lookup_ident(ident);
			if ns_lookup.is_some() {
				return ns_lookup;
			}
		}
		return None;
	}
	pub fn parse_attrs(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<AttributeList, ParserError> {
		let mut attrs = AttributeList::new();
		while self.state.consume_sym(ids, "[[")? {
			let attr_name = self.state.expect_ident(ids)?;
			let attr_value = if self.state.consume_sym(ids, "=")? {
				self.parse_expression(ids, curr_scope)?
			} else {
				Expression::new(ExprType::Null)
			};
			attrs.0.push(Attribute {name: attr_name, value: attr_value});
			self.state.expect_sym(ids, "]]")?;
		}
		Ok(attrs)
	}
	pub fn parse_template_decl(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<Vec<TemplateArg>, ParserError> {
		let mut args = Vec::new();
		if self.state.consume_kw(ids, constids::template)? {
			self.state.expect_sym(ids, "<")?;
			while !self.state.consume_sym(ids, ">")? {
				let attrs = self.parse_attrs(ids, curr_scope)?;
				if self.state.consume_kw(ids, constids::typename)? {
					let arg_name = self.state.expect_ident(ids)?;
					let mut arg_default = None;
					if self.state.consume_sym(ids, "=")? {
						arg_default = Some(self.parse_datatype(ids, curr_scope)?.ok_or_else(|| self.state.err(format!("expected data type")))?);
					}
					args.push(TemplateArg::typename(arg_name, arg_default, attrs));
				} else {
					let arg_type = self.parse_datatype(ids, curr_scope)?.ok_or_else(|| self.state.err(format!("expected data type")))?;
					let arg_name = self.state.expect_ident(ids)?;
					let mut arg_default = None;
					if self.state.consume_sym(ids, "=")? {
						arg_default = Some(self.parse_expression(ids, curr_scope)?);
					}
					args.push(TemplateArg::value(arg_name, arg_type, arg_default, attrs));
				}
				if !self.state.consume_sym(ids, ",")? {
					self.state.expect_sym(ids, ">")?;
					break;
				}
			}
		}
		Ok(args)
	}
	pub fn parse_statement(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<Option<Statement>, ParserError> {
		let attrs = self.parse_attrs(ids, curr_scope)?;
		if self.state.consume_kw(ids, constids::typedef)? {
			// typedef
		} else if self.state.consume_kw(ids, constids::using)? {
			// using 
		}
		Ok(None)
	}
	pub fn parse_template_vals(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<Vec<TemplateValue>, ParserError> {
		let mut vals = Vec::new();
		if self.state.consume_sym(ids, "<")? {
			while !self.state.consume_sym(ids, ">")? {
				// Try parse as a type
				self.state.enter_ambig();
				let typ = self.parse_datatype(ids, curr_scope)?;
				if let Some(typ) = typ {
					self.state.ambig_success(ids)?;
					vals.push(TemplateValue::Typ(typ))
				} else {
					self.state.ambig_failure(ids)?;
					vals.push(TemplateValue::Expr(self.parse_expression(ids, curr_scope)?))
				}
				if !self.state.consume_sym(ids, ",")? {
					self.state.expect_sym(ids, ">")?;
					break;
				}
			}
		}
		Ok(vals)
	}
	pub fn parse_integral_type(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<IntegerType, ParserError> {
		// TODO: this is a bit on the liberal side
		let mut width = Expression::from_u64(32, 32);
		let mut is_signed = Expression::from_u64(1, 1);
		loop {
			if self.state.check_kws(&[constids::signed, constids::unsigned]) {
				if self.state.consume_kw(ids, constids::signed)? { is_signed =  Expression::from_u64(1, 1); }
				else if self.state.consume_kw(ids, constids::unsigned)? { is_signed =  Expression::from_u64(0, 1); }
				// Could be a C-style type or a Meowality arb-precision int
				let tv = self.parse_template_vals(ids, curr_scope)?;
				if tv.len() > 1 {
					return Err(self.state.err(format!("integer types expect one template argument")));
				} else if tv.len() == 1 {
					width = tv[0].as_expr().ok_or_else(|| self.state.err(format!("integer types expect one template argument")))?;
				}
			} else if self.state.consume_kw(ids, constids::char)? {
				width = Expression::from_u64(8, 32);
			} else if self.state.consume_kw(ids, constids::short)? {
				width = Expression::from_u64(16, 32);
			} else if self.state.consume_kw(ids, constids::int)? {
				// no-op as 32 is the default anyway, and we want to support patterns like long int
			} else if self.state.consume_kw(ids, constids::long)? {
				width = Expression::from_u64(64, 32);
			} else {
				break;
			}
		}
		Ok(IntegerType{width, is_signed})
	}
	// This is a bit natty. Data type parsing can fail in two ways - either totally invalid syntax, or syntax that is potentially valid but definitely not a data type, and in some contexts we might need to try parsing the latter again as something else. We disambiguate between the two with a either an Err (total failure) or None (retry parse as expression).
	pub fn parse_datatype(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<Option<DataType>, ParserError> {
		let mut is_typename = false;
		let mut is_const = false;
		let mut is_static = false;
		// Parse qualifiers
		loop {
			if self.state.consume_kw(ids, constids::typename)? {
				is_typename = true;
			} else if self.state.consume_kw(ids, constids::r#const)? {
				is_const = true;
			} else if self.state.consume_kw(ids, constids::r#static)? {
				is_static = true;
			} else {
				break;
			}
		}
		// Basic type
		let mut dt : DataTypes = if self.state.consume_kw(ids, constids::auto)? {
			DataTypes::Auto
		} else if self.state.consume_kw(ids, constids::void)? {
			DataTypes::Void
		} else if self.state.check_kws(INTEGRAL_TYPES) {
			DataTypes::Integer(self.parse_integral_type(ids, curr_scope)?)
		} else if let Some(ident) = self.state.consume_ident(ids,)? {
			// is_typename forces identifier to be a type
			if is_typename || curr_scope.is_type(ident) {
				// TODO: template arguments
				DataTypes::User(UserType{name: ident, args: self.parse_template_vals(ids, curr_scope)?})
			} else {
				return Ok(None);
			}
		} else {
			return Ok(None);
		};
		let apply_mod = |d| DataType { is_const, is_static, typ: d };
		// Things that can follow
		loop {
			if self.state.consume_sym(ids, "[")? {
				let dims = self.parse_expression_list(ids, curr_scope, "]")?;
				self.state.expect_sym(ids, "]")?;
				dt = DataTypes::Array(ArrayType { base: Box::new(apply_mod(dt)), dims });
			} else if self.state.consume_sym(ids, "::")? {
				dt = DataTypes::ScopedType(Box::new(apply_mod(dt)), self.state.expect_ident(ids)?);
			} else if self.state.consume_sym(ids, "&")? {
				dt = DataTypes::Reference(Box::new(apply_mod(dt)))
			} else {
				break
			}
		}
		Ok(Some(apply_mod(dt)))
	}
	pub fn resolve_ident(&self, curr_scope: &dyn Scope, ident: IdString) -> Result<IdentifierType, ParserError> {
		self.lookup_ident(curr_scope, ident).ok_or_else(|| self.state.err(format!("unexpected identifier {}", ident)))
	}
	pub fn pop_op_stack(&mut self, op_stack: &mut Vec<OpStackItem>, expr_stack: &mut Vec<Expression>) -> Result<(), ParserError> {
		let op = op_stack.pop().ok_or_else(|| self.state.err(format!("operation stack underflow")))?;
		match op {
			OpStackItem::Op(o) => {
				let mut args = Vec::new();
				for _ in 0..o.arg_count() {
					args.push(expr_stack.pop().ok_or_else(|| self.state.err(format!("too few arguments for operator {}", o.token())))?);
				}
				args.reverse();
				expr_stack.push(Expression::new(ExprType::Op(o, args)));
			}
			_ => {}
		}
		Ok(())
	}
	pub fn parse_expression(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<Expression, ParserError> {
		use ExprType::*;
		// For our cursed shunting-yard-esque expression parsing
		let mut last_was_operator = true;
		let mut op_stack : Vec<OpStackItem> = Vec::new();
		let mut expr_stack : Vec<Expression> = Vec::new();
		loop {
			if let Some(tok) = self.state.consume_literal(ids)? {
				last_was_operator = false;
				match tok {
					Token::IntLiteral(bv) => {
						expr_stack.push(Expression::new(Literal(bv)));
					}
					_ => { return Err(self.state.err(format!("unsupported literal {:?}", tok))); }
				}
			} else if let Some(id) = self.state.consume_ident(ids)? {
				last_was_operator = false;
				// self.resolve_ident(curr_scope, id)?;
				expr_stack.push(Expression::new(Variable(id)));
			} else if self.state.consume_sym(ids, "(")? {
				if last_was_operator {
					// parentheses
					op_stack.push(OpStackItem::LParen);
				} else {
					// function call
					let target = expr_stack.pop().unwrap();
					let templ_vals = self.parse_template_vals(ids, curr_scope)?;
					expr_stack.push(Expression::new(Func(
						FuncCall {
							target: Box::new(target),
							targs: templ_vals,
							args: self.parse_expression_list(ids, curr_scope, ")")?
						}
					)));
					self.state.expect_sym(ids, ")")?;
				}
				last_was_operator = true;
			} else if self.state.check_sym(")") {
				while !op_stack.is_empty() && op_stack.last().cloned() != Some(OpStackItem::LParen) {
					self.pop_op_stack(&mut op_stack, &mut expr_stack)?;
				}
				if op_stack.is_empty() {
					// underflow, don't swallow bracket as something else is expecting it as a terminator
					break;
				} else {
					// end of potentially nested brackets, safe to swallow terminator
					self.state.expect_sym(ids, ")")?;
					assert_eq!(op_stack.pop(), Some(OpStackItem::LParen));
				}
				last_was_operator = false;
			} else if self.state.check_sym("{") {
				// initialiser list
				expr_stack.push(Expression::new(List(self.parse_expression_list(ids, curr_scope, "}")?)));
				self.state.expect_sym(ids, "}")?;
				last_was_operator = false;
			} else if let Some(op_sym) = self.state.consume_any_sym(ids, Operator::SYMBOLS)? {
				let op = if last_was_operator {
					// unary prefix
					Operator::lookup(op_sym, 1, false)
				} else {
					// binary or unary postfix
					Operator::lookup(op_sym, 2, false).or_else(|| Operator::lookup(op_sym, 1, true))
				}.unwrap();
				// shunting yard
				while !op_stack.is_empty() {
					match op_stack.last().unwrap() {
						OpStackItem::Op(top) => {
							if (op.is_right_assoc() && op.precedence() > top.precedence())
								|| (!op.is_right_assoc() && op.precedence() >= top.precedence()) {
								self.pop_op_stack(&mut op_stack, &mut expr_stack)?;
							} else {
								break;
							}
						}
						_ => break,
					}
				}
				op_stack.push(OpStackItem::Op(op));
				last_was_operator = true;
			} else {
				break;
			}
		}
		Err(self.state.err(format!("unable to parse expression")))
	}
	pub fn parse_expression_list(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope, terminator: &'static str) -> Result<Vec<Expression>, ParserError> {
		let mut exprs = Vec::new();
		while !self.state.check_sym(terminator) {
			exprs.push(self.parse_expression(ids, curr_scope)?);
			if !self.state.consume_sym(ids, ",")? {
				break;
			}
		}
		Ok(exprs)
	}
}

#[cfg(test)]
pub mod test {
	use crate::parser::Tokeniser;
	use super::*;
	fn setup(s: &'static str) -> Result<(IdStringDb, Parser<std::str::Chars>, Namespace), ParserError> {
		let mut ids = IdStringDb::new();
		constids::do_ids_init(&mut ids);
		let tok = Tokeniser::new(ids.id("<test>"), s.chars());
		let ps = ParserState::new(tok, &mut ids)?;
		let p = Parser::new(ps);
		let root = Namespace::new(None, AttributeList::new(), SrcInfo::default());
		Ok((ids, p, root))
	}

	#[test]
	fn prim_types() -> Result<(), ParserError> {
		use DataTypes::*;
		let (mut ids, mut p, r) = setup("char; unsigned<33>; unsigned short int; signed;")?;
		let exp_types = &[(8, true), (33, false), (16, false), (32, true)];
		for (width, is_signed) in exp_types {
			let dt = p.parse_datatype(&mut ids, &r)?.unwrap();
			match dt.typ {
				Integer(i) => {
					assert_eq!(i.width.as_u64(), Some(*width));
					assert_eq!(i.is_signed.as_u64().map(|x| x != 0), Some(*is_signed));
				}
				_ => assert!(false)
			}
			p.state.expect_sym(&mut ids, ";")?;
		}
		Ok(())
	}

	#[test]
	fn attrs() -> Result<(), ParserError> {
		let (mut ids, mut p, r) = setup("[[attr=11]] [[another_attr]]")?;
		assert_eq!(p.parse_attrs(&mut ids, &r)?, AttributeList(vec![
			Attribute { name: ids.id("attr"), value: Expression::from_u64(11, 64) },
			Attribute { name: ids.id("another_attr"), value: Expression::new(ExprType::Null) }
		]));
		Ok(())
	}

	#[test]
	fn complex_types() -> Result<(), ParserError> {
		use DataTypes::*;
		let (mut ids, mut p, r) = setup("typename our_struct<unsigned<19>, our_const>")?;
		let dt = p.parse_datatype(&mut ids, &r)?.unwrap();
		match dt.typ {
			User(ut) => {
				assert_eq!(ut.name, ids.id("our_struct"));
				assert_eq!(ut.args[0].as_type(), Some( DataType { typ: Integer( IntegerType { width: Expression::from_u64(19, 64), is_signed: Expression::from_u64(0, 1) }), is_const: false, is_static: false } ));
				assert_eq!(ut.args[1].as_expr(), Some( Expression::new(ExprType::Variable(ids.id("our_const"))) ));
			},
			_ => assert!(false)
		}
		Ok(())
	}
}
