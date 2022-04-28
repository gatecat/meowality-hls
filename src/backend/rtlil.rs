use crate::core::{IdStringDb, IdString, BasicOp};
use crate::design::PortDir;
use crate::backend::low_netlist::*;

use std::io::{Write, Result};
use std::fs::File;

struct RTLILBackend<'a> {
	nl: LowNetlist,
	f: File,
	ids: &'a IdStringDb,
}

impl <'a> RTLILBackend<'a> {
	pub fn run(nl: LowNetlist, filename: &str, ids: &IdStringDb) -> Result<()> {
		let mut ctx = RTLILBackend {nl: nl, f: File::create(filename)?, ids: ids};
		ctx.write_design()?;
		Ok(())
	}
	fn s(&self, id: IdString) -> String {
		let st = id.str(self.ids);
		if st.chars().next() == Some('$') {
			st.to_string()
		} else {
			format!("\\{}", st)
		}
	}
	fn write_wires(&mut self) -> Result<()> {
		let mut port_idx = 0;
		for (name, data) in self.nl.nodes.iter() {
			write!(self.f, "wire width {} ", data.typ.width)?;
			if let Some(dir) = data.dir {
				match dir {
					PortDir::Input => write!(self.f, "input ")?,
					PortDir::Output => write!(self.f, "output ")?,
				}
				write!(self.f, "{} ", port_idx)?;
				port_idx += 1;
			}
			if data.typ.is_signed { write!(self.f, "signed ")?; }
			writeln!(self.f, "{}", self.s(*name))?;
		}
		Ok(())
	}

	fn get_yosys_cell(&self, op: BasicOp, _is_signed: bool) -> &'static str {
		match op {
			BasicOp::Add => { "$add" },
			BasicOp::Sub => { "$sub" },
			// ...
			_ => unimplemented!(),
		}
	}

	fn write_prims(&mut self) -> Result<()> {
		for (name, data) in self.nl.nodes.iter() {
			match &data.value {
				LowPrim::Null | LowPrim::Input => {},
				LowPrim::Op {ty, a, b} => {
					writeln!(self.f, "  cell {} {}_op", self.get_yosys_cell(*ty, data.typ.is_signed), self.s(*name))?;
					let a_node = self.nl.nodes.get(a).unwrap();
					writeln!(self.f, "     parameter \\A_SIGNED {}", a_node.typ.is_signed)?;
					writeln!(self.f, "     parameter \\A_WIDTH {}", a_node.typ.width)?;
					if *b != IdString::NONE {
						let b_node = self.nl.nodes.get(b).unwrap();
						writeln!(self.f, "     parameter \\B_SIGNED {}", b_node.typ.is_signed)?;
						writeln!(self.f, "     parameter \\B_WIDTH {}", b_node.typ.width)?;
					} 
					writeln!(self.f, "     parameter \\Y_WIDTH {}", data.typ.width)?;
					writeln!(self.f, "     connect \\A {}", self.s(*a))?;
					if *b != IdString::NONE { writeln!(self.f, "     connect \\B {}", self.s(*b))?; }
					writeln!(self.f, "     connect \\Y {}", self.s(*name))?;
				},
				LowPrim::Const { val } => {
					writeln!(self.f, "  connect {} {}", self.s(*name), val.to_string())?;
				},
				_ => unimplemented!(),
			}
		}
		Ok(())
	}

	fn write_design(&mut self) -> Result<()> {
		writeln!(self.f, "module {}", self.s(self.nl.name))?;
		self.write_wires()?;
		self.write_prims()?;
		writeln!(self.f, "end")?;
		Ok(())
	}
}