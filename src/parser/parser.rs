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
	// the result of the parser is the root namespace
	root: Namespace,
}

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
			root: Namespace::new(None, AttributeList::new(), SrcInfo::default()),
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
						arg_default = Some(self.parse_datatype(ids, curr_scope)?);
					}
					args.push(TemplateArg::typename(arg_name, arg_default, attrs));
				} else {
					let arg_type = self.parse_datatype(ids, curr_scope)?;
					let arg_name = self.state.expect_ident(ids)?;
					let mut arg_default = None;
					if self.state.consume_sym(ids, "=")? {
						arg_default = Some(self.parse_expression(ids, curr_scope)?);
					}
					args.push(TemplateArg::value(arg_name, arg_type, arg_default, attrs));
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
	pub fn parse_integral_type(&mut self, ids: &mut IdStringDb, _curr_scope: &dyn Scope) -> Result<IntegerType, ParserError> {
		unimplemented!()
	}
	pub fn parse_datatype(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<DataType, ParserError> {
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
		} else if self.state.check_kws(INTEGRAL_TYPES) {
			DataTypes::Integer(self.parse_integral_type(ids, curr_scope)?)
		} else if let Some(ident) = self.state.consume_ident(ids,)? {
			// is_typename forces identifier to be a type
			if is_typename || curr_scope.is_type(ident) {
				// TODO: template arguments
				DataTypes::User(UserType{name: ident, args: FxHashMap::default()})
			} else {
				return Err(self.state.err(format!("expected data type, found {}", ident)));
			}
		} else {
			return Err(self.state.err(format!("failed to parse data type")));
		};
		// Things that can follow
		let apply_mod = |d| DataType { is_const, is_static, typ: d };
		Ok(apply_mod(dt))
	}
	pub fn resolve_ident(&self, curr_scope: &dyn Scope, ident: IdString) -> Result<IdentifierType, ParserError> {
		self.lookup_ident(curr_scope, ident).ok_or_else(|| self.state.err(format!("unexpected identifier {}", ident)))
	}
	pub fn parse_expression(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<Expression, ParserError> {
		use ExprType::*;
		if let Some(tok) = self.state.consume_literal(ids)? {
			match tok {
				Token::IntLiteral(bv) => {
					return Ok(Expression::new(Literal(bv)));
				}
				_ => { return Err(self.state.err(format!("unsupported literal {:?}", tok))); }
			}
		} else if let Some(id) = self.state.consume_ident(ids)? {
			self.resolve_ident(curr_scope, id)?;
			return Ok(Expression::new(Variable(id)));
		}
		Err(self.state.err(format!("unable to parse expression")))
	}
}