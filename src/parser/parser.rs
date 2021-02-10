use crate::ast::*;
use crate::core::{IdString, IdStringDb};
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
	pub fn resolve_ident(&self, curr_scope: &dyn Scope, ident: IdString) -> Result<IdentifierType, ParserError> {
		self.lookup_ident(curr_scope, ident).ok_or_else(|| self.state.err(format!("unexpected identifier {}", ident)))
	}
	pub fn parse_expression(&mut self, ids: &mut IdStringDb, curr_scope: &dyn Scope) -> Result<Expression, ParserError> {
		use ExprType::*;
		if let Some(tok) = self.state.consume_literal(ids)? {
			match tok {
				Token::IntLiteral(bv) => {
					return Ok(Expression::new(Literal(bv), AttributeList::new(), SrcInfo::default()));
				}
				_ => { return Err(self.state.err(format!("unsupported literal {:?}", tok))); }
			}
		} else if let Some(id) = self.state.consume_ident(ids)? {
			self.resolve_ident(curr_scope, id)?;
			return Ok(Expression::new(Variable(id), AttributeList::new(), SrcInfo::default()));
		}
		Err(self.state.err(format!("unable to parse expression")))
	}
}