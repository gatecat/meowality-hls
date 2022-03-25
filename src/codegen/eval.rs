use crate::BasicOp;
use crate::ast::{SrcInfo, Expression, Operator};
use crate::core::constids;
use crate::codegen::state::*;
use crate::codegen::{ResolvedTypes, Value};
use crate::design::PrimitiveType;

struct Eval <'a> {
	pub st: &'a mut GenState<'a>,
	pub sc: &'a mut GenScope<'a>,
	pub is_const: bool,
}

impl <'a> Eval<'a> {
	fn op_value(&mut self, src: SrcInfo, op: BasicOp, args: Vec<Value>) -> Result<Value, CodegenError> {
		let mut types = Vec::new();
		let mut const_vals = Vec::new();
		let mut fully_const = true;
		for arg in args.iter() {
			let val_type = arg.to_type(self.st).unwrap(); // TODO: why could this ever not resolve?
			if let ResolvedTypes::Integer(it) = val_type.typ {
				types.push(it);
			} else {
				return Err(CodegenError(src, format!("non-scalar value {:?} passed to operator {:?}", arg, op)));
			}
			if let Value::Constant(c) = arg {
				const_vals.push(c.clone());
			} else {
				fully_const = false;
			}
		}
		if fully_const {
			// Constant fold
			Ok(Value::Constant(op.apply(&const_vals)))
		} else {
			// Create a node
			let res_type = op.result_type(&types);
			let prim_name = self.st.des.auto_id(self.st.ids);
			let prim = self.st.des.add_prim(prim_name, PrimitiveType::BasicOp(op), src).unwrap();
			let input_names = &[constids::A, constids::B];
			for i in 0..args.len() {
				let node = self.st.get_node(&args[i], src);
				self.st.des.add_prim_input(prim, input_names[i], node).unwrap();
			}
			let node_name = self.st.des.auto_id(self.st.ids); // todo: more descriptive
			Ok(Value::from_node(self.st.des.add_node(node_name, res_type, src, prim, constids::Q).unwrap()))
		}
	}
	fn eval_oper(&mut self, src: SrcInfo, ty: Operator, args: Vec<Value>) -> Result<Value, CodegenError> {
		use crate::ast::Operator::*;
		match ty {
			Add => self.op_value(src, BasicOp::Add, args),
			_ => unimplemented!()
		}
	}
	fn eval_expr(&mut self, expr: &Expression) -> Result<Value, CodegenError> {
		use crate::ast::ExprType::*;
		match &expr.ty { 
			Literal(x) => Ok(Value::Constant(x.clone())),
			Variable(v) => {
				let var_idx = self.sc.lookup_var(*v).unwrap_or_err(|| CodegenError(expr.src, format!("unable to resolve variable {}", v)))?;
				let value = self.st.vars.get(var_idx).value.clone();
				if self.is_const && !value.is_fully_const() {
					Err(CodegenError(expr.src, format!("attempting to use non-constant value {:?} in constant ctx", v)))
				} else {
					Ok(value)
				}
			},
			Op(ty, args) => {
				let mut mapped_args = Vec::new();
				for a in args.iter() { mapped_args.push(self.eval_expr(a)?); }
				self.eval_oper(expr.src, *ty, mapped_args)
			},
			Null => Ok(Value::Void),
			_ => {unimplemented!()}
		}
	}
}
