//! A simplified LLVM-style IR code generator.

use std::fmt;

/// Virtual register ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(u32);

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%r{}", self.0)
    }
}

/// IR instruction opcodes.
#[derive(Debug, Clone, PartialEq)]
pub enum IrOp {
    Add { dst: Reg, lhs: Reg, rhs: Reg },
    Sub { dst: Reg, lhs: Reg, rhs: Reg },
    Mul { dst: Reg, lhs: Reg, rhs: Reg },
    Div { dst: Reg, lhs: Reg, rhs: Reg },
    Const { dst: Reg, value: i64 },
    Load { dst: Reg, slot: usize },
    Store { slot: usize, src: Reg },
    Ret { val: Reg },
    Call { dst: Reg, func: String, args: Vec<Reg> },
    Phi { dst: Reg, incoming: Vec<(Reg, String)> },
}

/// A basic block of IR instructions.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub instrs: Vec<IrOp>,
}

impl BasicBlock {
    pub fn new(label: &str) -> Self {
        BasicBlock {
            label: label.to_string(),
            instrs: Vec::new(),
        }
    }
}

/// An IR function.
#[derive(Debug, Clone)]
pub struct IrFunc {
    pub name: String,
    pub params: Vec<Reg>,
    pub blocks: Vec<BasicBlock>,
    next_reg: u32,
}

impl IrFunc {
    pub fn new(name: &str, param_count: usize) -> Self {
        let params: Vec<Reg> = (0..param_count).map(|i| Reg(i as u32)).collect();
        let next_reg = param_count as u32;
        IrFunc {
            name: name.to_string(),
            params,
            blocks: vec![BasicBlock::new("entry")],
            next_reg,
        }
    }

    /// Allocate a fresh register.
    pub fn alloc_reg(&mut self) -> Reg {
        let r = Reg(self.next_reg);
        self.next_reg += 1;
        r
    }

    /// Append an instruction to the current block.
    pub fn emit(&mut self, op: IrOp) {
        self.blocks.last_mut().unwrap().instrs.push(op);
    }

    /// Create a new basic block and return its index.
    pub fn new_block(&mut self, label: &str) -> usize {
        let idx = self.blocks.len();
        self.blocks.push(BasicBlock::new(label));
        idx
    }
}

impl fmt::Display for IrFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self.params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
        writeln!(f, "fn {}({}) {{", self.name, params)?;
        for block in &self.blocks {
            writeln!(f, "  {}:", block.label)?;
            for instr in &block.instrs {
                writeln!(f, "    {}", format_ir(instr))?;
            }
        }
        write!(f, "}}")
    }
}

fn format_ir(op: &IrOp) -> String {
    match op {
        IrOp::Add { dst, lhs, rhs } => format!("{} = add {} {}", dst, lhs, rhs),
        IrOp::Sub { dst, lhs, rhs } => format!("{} = sub {} {}", dst, lhs, rhs),
        IrOp::Mul { dst, lhs, rhs } => format!("{} = mul {} {}", dst, lhs, rhs),
        IrOp::Div { dst, lhs, rhs } => format!("{} = div {} {}", dst, lhs, rhs),
        IrOp::Const { dst, value } => format!("{} = const {}", dst, value),
        IrOp::Load { dst, slot } => format!("{} = load [{}]", dst, slot),
        IrOp::Store { slot, src } => format!("store [{}] {}", slot, src),
        IrOp::Ret { val } => format!("ret {}", val),
        IrOp::Call { dst, func, args } => {
            let a = args.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", ");
            format!("{} = call {}({})", dst, func, a)
        }
        IrOp::Phi { dst, incoming } => {
            let pairs: Vec<String> = incoming.iter().map(|(r, l)| format!("[{}, {}]", r, l)).collect();
            format!("{} = phi {}", dst, pairs.join(", "))
        }
    }
}

/// A module containing multiple functions.
pub struct IrModule {
    pub name: String,
    pub functions: Vec<IrFunc>,
}

impl IrModule {
    pub fn new(name: &str) -> Self {
        IrModule { name: name.to_string(), functions: Vec::new() }
    }

    pub fn add_func(&mut self, f: IrFunc) {
        self.functions.push(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_ir() {
        let mut func = IrFunc::new("add", 2);
        let dst = func.alloc_reg();
        func.emit(IrOp::Add {
            dst,
            lhs: func.params[0],
            rhs: func.params[1],
        });
        func.emit(IrOp::Ret { val: dst });
        let s = func.to_string();
        assert!(s.contains("fn add"));
        assert!(s.contains("ret"));
    }
}
