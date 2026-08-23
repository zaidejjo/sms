# SMS - Smart Math Solver

A high-performance, command-line equation solver and mathematical utility written in Rust. Designed for sub-millisecond solving times, complex root finding, linear systems, and series evaluations.

## Features

- **Equation Solving**: Numerical and analytical solving for polynomial, trigonometric, exponential, logarithmic, and complex equations.
- **Linear Systems**: Matrix equation solver using Gaussian elimination.
- **Series Computation**: High-speed summation ($\sum$) and product ($\prod$) evaluations.
- **AI-Powered Solver**: Heuristic gradient descent solver for non-standard or continuous equations.
- **Constants Support**: Built-in constants ($\pi$, $e$, $\phi$, $	au$, $\sqrt{2}$, $\sqrt{3}$).
- **Fraction Display**: Automatic conversion of decimals to simplified fractions.
- **Multi-Format Export**: Export results directly to JSON, CSV, or LaTeX.
- **Complex Numbers**: Full support for real and complex roots.
- **Rust Crate Support**: Use directly as a library in downstream Rust projects.

---

## Installation

### From AUR (Arch Linux)

```bash
yay -S sms
# or
paru -S sms
```

### From Source

Ensure you have Rust 1.70 or higher installed.

```bash
git clone https://github.com/zaidejjo/sms.git
cd sms
cargo build --release
sudo cp target/release/sms /usr/local/bin/
```

### As a Library

Add `sms` to your `Cargo.toml`:

```toml
[dependencies]
sms = "0.1.0"
```

---

## Quick Start

Launch the interactive CLI:

```bash
sms
```

---

## Examples

```text
> x^2 - 4
  1. x = 2
  2. x = -2
  2 solutions found
  Time: 4.234ms

> sin(π/2)
  = 1  (fraction: 1/1)
  Time: 0.123ms

> 1/2 + 1/3
  = 0.833333  (fraction: 5/6)
  Time: 0.045ms

> matrix [[2,3],[4,-1]] [8,6]
  x1 = 2
  x2 = 4/3

> sum i^2, i=1..10
  = 385

> product i, i=1..5
  = 120

> ai x^5 + x^4 - 10*x^3 + 5*x^2 - 2*x + 3
  x = 1.234567  (error: 2.34e-6, iter: 4521)

> export
  Exported: result.json, result.csv, result.tex
```

---

## Command Reference

| Command | Description |
| :--- | :--- |
| `<equation>` | Solve any equation (e.g., `x^2 - 4`, `sin(x) = 0.5`) |
| `matrix A b` | Solve linear system $Ax = b$ |
| `sum expr, var=a..b` | Compute series sum |
| `product expr, var=a..b` | Compute series product |
| `ai <equation>` | AI-powered gradient descent solver |
| `plot var,min,max` | Plot function in terminal |
| `export` | Export last results to JSON, CSV, LaTeX |
| `history` | Show command history |
| `clear` / `cls` | Clear terminal screen |
| `quit` / `exit` | Exit REPL |

---

## Supported Functions

- **Trigonometric**: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`
- **Hyperbolic**: `sinh`, `cosh`, `tanh`
- **Logarithmic**: `ln`, `log(x, base)`
- **Exponential**: `exp`, `^`
- **Other**: `sqrt`, `abs`
- **Constants**: $\pi$, $e$, $\phi$, $	au$, $\sqrt{2}$, $\sqrt{3}$

---

## Performance

SMS is optimized for high-speed computation:

| Equation | Benchmark Avg |
| :--- | :--- |
| `x^2 - 4` | ~4.0 ms |
| `(x-1)^15` | ~1.0 ms |
| `(x-1)^30` | ~1.0 ms |
| Complex equations | ~50–100 ms |

---

## Library Usage

```rust
use sms::{Parser, EquationSolver};

fn main() {
    let mut parser = Parser::new("x^2 - 4");
    let expr = parser.parse_equation();
    
    let solver = EquationSolver::new();
    let (real, complex) = solver.find_all_roots(&expr, 'x');
    
    println!("Real roots: {:?}", real);
    println!("Complex roots: {:?}", complex);
}
```

---

## Project Structure

```text
sms/
├── src/
│   ├── main.rs          # CLI interface and REPL
│   ├── expr.rs          # Expression AST
│   ├── parser.rs        # Expression parser
│   ├── solver.rs        # Equation solver
│   ├── matrix.rs        # Matrix operations
│   ├── series.rs        # Series computation
│   ├── ai.rs            # AI-powered gradient descent solver
│   ├── constants.rs     # Mathematical constants
│   ├── fractions.rs     # Fraction handling
│   └── export.rs        # Export functionality
├── Cargo.toml
└── README.md
```

---

## Contributing

1. Fork the repository (`https://github.com/zaidejjo/sms`)
2. Create your feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing`)
5. Open a Pull Request

---

## License

Distributed under the [MIT License](LICENSE).
