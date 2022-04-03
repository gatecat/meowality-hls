use crate::{BasicOp, BitVector, IdStringDb, IdString};
use crate::ast::{SrcInfo, Expression, Statement, Operator, IODir};
use crate::core::constids;
use crate::codegen::state::*;
use crate::codegen::{ResolvedType, ResolvedTypes, LValue, RValue, Variable};
use crate::design::{PortDir, PrimitiveType};

pub struct Eval <'a> {
	pub st: GenState<'a>,
	pub is_const: bool,
}

impl <'a> Eval<'a> {
	pub fn op_value(&mut self, src: SrcInfo, op: BasicOp, args: &[Expression]) -> Result<RValue, CodegenError> {
		let mut types = Vec::new();
		let mut const_vals = Vec::new();
		let mut fully_const = true;
		let mut mapped_args = Vec::new();
		for a in args.iter() { mapped_args.push(self.eval_rvalue(a)?); }
		for arg in mapped_args.iter() {
			let val_type = arg.to_type(&self.st).unwrap(); // TODO: why could this ever not resolve?
			if let ResolvedTypes::Integer(it) = val_type.typ {
				types.push(it);
			} else {
				return Err(CodegenError(src, format!("non-scalar value {:?} passed to operator {:?}", arg, op)));
			}
			if let RValue::Constant(c) = arg {
				const_vals.push(c.clone());
			} else {
				fully_const = false;
			}
		}
		if fully_const {
			// Constant fold
			Ok(RValue::Constant(op.apply(&const_vals)))
		} else {
			// Create a node
			let res_type = op.result_type(&types);
			let prim_name = self.st.des.auto_id(self.st.ids);
			let prim = self.st.des.add_prim(prim_name, PrimitiveType::BasicOp(op), src).unwrap();
			let input_names = &[constids::A, constids::B];
			for i in 0..mapped_args.len() {
				let node = self.st.get_node(&mapped_args[i], src);
				self.st.des.add_prim_input(prim, input_names[i], node).unwrap();
			}
			let node_name = self.st.des.auto_id(self.st.ids); // todo: more descriptive
			Ok(RValue::from_node(self.st.des.add_node(node_name, res_type, src, prim, constids::Q).unwrap()))
		}
	}
	pub fn assign(&mut self, src: SrcInfo, lv: LValue, rv: RValue) -> Result<RValue, CodegenError> {
		self.st.assign_variable(lv.var, &lv.path, &rv, src);
		Ok(rv)
	}
	pub fn eval_oper(&mut self, src: SrcInfo, ty: Operator, args: &[Expression]) -> Result<RValue, CodegenError> {
		use crate::ast::Operator::*;
		match ty {
			Add => self.op_value(src, BasicOp::Add, args),
			Assign => {
				let lv = self.eval_lvalue(&args[0])?;
				let rv = self.eval_rvalue(&args[1])?;
				self.assign(src, lv, rv)
			}
			_ => unimplemented!()
		}
	}
	pub fn eval_rvalue(&mut self, expr: &Expression) -> Result<RValue, CodegenError> {
		use crate::ast::ExprType::*;
		match &expr.ty { 
			Literal(x) => Ok(RValue::Constant(x.clone())),
			Variable(v) => {
				let var_idx = self.st.lookup_var(*v).unwrap_or_err(|| CodegenError(expr.src, format!("unable to resolve variable {}", v)))?;
				let value = self.st.vars.get(var_idx).value.clone();
				if self.is_const && !value.is_fully_const() {
					Err(CodegenError(expr.src, format!("attempting to use non-constant value {:?} in constant ctx", v)))
				} else {
					Ok(value)
				}
			},
			Op(ty, args) => {
				self.eval_oper(expr.src, *ty, args)
			},
			Null => Ok(RValue::Void),
			_ => {unimplemented!()}
		}
	}
	pub fn eval_lvalue(&mut self, expr: &Expression) -> Result<LValue, CodegenError> {
		use crate::ast::ExprType::*;
		match &expr.ty { 
			Variable(v) => {
				let var_idx = self.st.lookup_var(*v).unwrap_or_err(|| CodegenError(expr.src, format!("unable to resolve variable {}", v)))?;
				Ok(LValue::from_var(var_idx))
			},
			_ => {Err(CodegenError(expr.src, format!("{:?} is not a valid l-value", expr)))}
		}
	}
	pub fn const_eval(&mut self, expr: &Expression) -> Result<RValue, CodegenError> {
		let old_is_const = self.is_const;
		self.is_const = true;
		let result = self.eval_rvalue(expr);
		self.is_const = old_is_const;
		result
	}
	pub fn const_eval_scalar(&mut self, expr: &Expression) -> Result<BitVector, CodegenError> {
		let result = self.const_eval(expr)?;
		if let RValue::Constant(c) = result {
			Ok(c)
		} else {
			Err(CodegenError(expr.src, format!("expected scalar constant got {:?}", result)))
		}
	}
	pub fn eval_st(&mut self, st: &Statement) -> Result<(), CodegenError> {
		use crate::ast::StatementType::*;
		match &st.ty {
			Null => {},
			Var(v) => {
				let var_init = if let Some(i) = &v.init {
					self.eval_rvalue(i)?
				} else {
					RValue::Void
				};
				let var_type = ResolvedType::do_resolve(self, &v.ty)?;
				let var_idx = self.st.vars.add(Variable {name: v.name, typ: var_type, value: var_init});
				self.st.scope().var_map.insert(v.name, var_idx);
			},
			Block(b) => {
				self.st.push_scope();
				for b_st in b.iter() {
					self.eval_st(b_st)?;
				}
				self.st.pop_scope();
			}
			If(ifs) => {
				let eval_cond = self.eval_rvalue(&ifs.cond)?;
				if let RValue::Constant(c) = eval_cond {
					// const eval if
					if c.as_bool() {
						self.eval_st(&ifs.if_true)?;
					} else if let Some(fls) = &ifs.if_false {
						self.eval_st(fls)?;
					}
				} else {
					if ifs.is_meta {
						return Err(CodegenError(st.src, format!("expected constant condition for 'if constexpr' got {:?}", eval_cond)));
					}
					if let RValue::Node(n) = eval_cond {
						self.st.push_cond(n, false);
						self.eval_st(&ifs.if_true)?;
						self.st.pop_cond();
						if let Some(fls) = &ifs.if_false {
							self.st.push_cond(n, true);
							self.eval_st(fls)?;
							self.st.pop_cond();
						}
					} else {
						return Err(CodegenError(st.src, format!("expected scalar condition for 'if constexpr' got {:?}", eval_cond)));
					}
				}
			},
			Expr(e) => {
				self.eval_rvalue(e)?;
			}
			_ => {unimplemented!()}
		}
		Ok(())
	}
	pub fn eval_mod(&mut self, m: &crate::ast::Module) -> Result<(), CodegenError> {
		for port in m.ports.iter() {
			match &port.dir {
				IODir::Input => {
					let ty = ResolvedType::do_resolve(self, &port.arg_type)?;
					let n = self.st.des.add_port(port.name, ty.pack()?, PortDir::Input).unwrap();
					let var_idx = self.st.vars.add(Variable {name: port.name, typ: ty, value: RValue::Node(n)});
					self.st.scope().var_map.insert(port.name, var_idx);
				}
				_ => {},
			}
		};
		self.eval_st(&m.content)?;
		Ok(())
	}
	pub fn init(ids: &'a mut IdStringDb, m: &crate::ast::Module) -> Self {
		let state = GenState::new(ids, m.name);
		Self {
			st: state,
			is_const: false,
		}
	}
}
