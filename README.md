# Codegen LLVM — SSA-Style Intermediate Representation Code Generator

**codegen-llvm** is a simplified LLVM IR-style code generator that constructs Static Single Assignment (SSA) intermediate representations from program ASTs. It models the key abstractions of compiler back-ends: virtual registers, basic blocks, φ-nodes (phi instructions), and function-level control flow.

## Why It Matters

Every compiler that targets LLVM goes through an IR generation phase — translating source-level constructs into the LLVM IR that LLVM's optimization passes and code generators consume. Understanding this layer is critical for anyone building a programming language, a JIT compiler, or a static analysis tool. The SSA form (where every variable is assigned exactly once) is the backbone of modern compiler optimization: it makes dominance explicit, simplifies liveness analysis, and enables powerful transformations like constant propagation, common subexpression elimination, and loop-invariant code motion. Even if you never write an LLVM backend, the abstractions here — registers, blocks, phi nodes — appear in WebAssembly, Cranelift, GCC's GIMPLE, and every modern JIT.

## How It Works

### SSA Form

In **Static Single Assignment** form, each virtual register is defined exactly once. If a variable is reassigned in the source program, the IR creates a fresh register:

```
Source:          SSA:
  x = 1            %r0 = const 1        ; x₁
  x = 2            %r1 = const 2        ; x₂
  y = x            %r2 = %r1            ; y = x₂ (latest)
```

### Φ-Nodes

When control flow merges (e.g., after an if-then-else), a φ-node selects between incoming values based on which predecessor block was executed:

```
  %r5 = phi [%r1, entry], [%r3, loop_body]
```

This says: `%r5` takes the value of `%r1` if we arrived from `entry`, or `%r3` if we arrived from `loop_body`.

### Instruction Set

The IR models these operations:

| Opcode | Form | Description |
|---|---|---|
| `Const` | `dst = const N` | Load immediate constant |
| `Add/Sub/Mul/Div` | `dst = op lhs, rhs` | Binary arithmetic |
| `Load/Store` | `dst = load [slot]` | Memory access (stack slots) |
| `Call` | `dst = call func(args)` | Function invocation |
| `Phi` | `dst = phi [val, label], ...` | SSA merge |
| `Ret` | `ret val` | Return from function |

### Register Allocation

Registers are allocated via a monotonically increasing counter — `alloc_reg()` returns a fresh `Reg(n)` and increments `n`. This is "infinite register" SSA, matching LLVM's model. Real register allocation (mapping virtual registers to physical CPU registers) happens in a later pass, typically via graph coloring or linear scan.

**Complexity**: All operations are `O(1)` — `alloc_reg`, `emit`, `new_block`. Pretty-printing is `O(total instructions)`.

## Quick Start

```rust
use codegen_llvm::{IrFunc, IrOp, IrModule, BasicBlock};

// Build: fn add(a, b) { return a + b; }
let mut func = IrFunc::new("add", 2);
let dst = func.alloc_reg();
func.emit(IrOp::Add {
    dst,
    lhs: func.params[0],
    rhs: func.params[1],
});
func.emit(IrOp::Ret { val: dst });

// Build a module
let mut module = IrModule::new("my_module");
module.add_func(func);

// Pretty-print
println!("{}", module.functions[0]);
// Output:
// fn add(%r0, %r1) {
//   entry:
//     %r2 = add %r0 %r1
//     ret %r2
// }
```

## API

| Type | Description |
|---|---|
| `IrModule` | A compilation unit containing multiple functions. |
| `IrFunc` | An IR function: name, params, basic blocks, register allocator. |
| `BasicBlock` | A labeled sequence of `IrOp` instructions. |
| `IrOp` | Instruction enum: `Add, Sub, Mul, Div, Const, Load, Store, Ret, Call, Phi`. |
| `Reg` | A virtual register handle (`%rN`). |
| `IrFunc::alloc_reg()` | Allocate a fresh virtual register. |
| `IrFunc::emit(op)` | Append an instruction to the current block. |
| `IrFunc::new_block(label)` | Create a new basic block, return its index. |

## Architecture Notes

codegen-llvm is the IR generation layer on the γ (generation/synthesis) side of γ + η = C. It takes high-level program representations and lowers them to SSA IR that can be consumed by optimization passes and code generators. In SuperInstance, it bridges the frontend parser and the backend optimizer. See [SuperInstance Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. Cytron, R. et al. (1991). *Efficiently Computing Static Single Assignment Form and the Control Dependence Graph*. ACM TOPLAS 13(4), 451–490. — The original SSA construction algorithm.
2. LLVM Language Reference Manual. <https://llvm.org/docs/LangRef.html> — Canonical IR specification.
3. Cooper, K. D. & Torczon, L. (2011). *Engineering a Compiler* (2nd ed.), Ch. 8–9. Morgan Kaufmann.

## License

MIT
