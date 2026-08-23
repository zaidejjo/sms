# SMS Feature Implementation Plan

## Overview
4 major features, ~6-8 weeks total (part-time), with clear dependencies and milestones.

---

## Phase 1: Foundation (Week 1-2)
**Goal**: Prepare codebase for extensions without breaking existing CLI.

### Tasks
- [ ] **Modularize public API** — Create `lib.rs` re-exports, stabilize `Expr`, `Parser`, `EquationSolver` interfaces
- [ ] **Add `sms-core` crate** — Split into `sms-core` (library) + `sms-cli` (binary) for WASM/TUI reuse
- [ ] **Property-based tests** — Add `proptest` for parser/solver correctness (catch regressions early)
- [ ] **Benchmark suite** — `cargo bench` with `criterion` for performance tracking

### Deliverable
```
sms/
├── Cargo.toml (workspace)
├── sms-core/          # NEW: library crate
│   ├── src/lib.rs
│   ├── expr.rs
│   ├── parser.rs
│   ├── solver.rs
│   ├── matrix.rs
│   ├── series.rs
│   ├── ai.rs
│   ├── constants.rs
│   ├── fractions.rs
│   ├── export.rs
│   └── symbolic.rs    # NEW: Phase 2
├── sms-cli/           # NEW: binary crate
│   └── src/main.rs
└── sms-tui/           # NEW: Phase 3
└── sms-wasm/          # NEW: Phase 5
```

---

## Phase 2: Symbolic Algebra Engine (Week 2-4)
**Goal**: Full symbolic manipulation — simplify, expand, factor, derivative, integrate.

### Architecture
```
sms-core/src/symbolic/
├── mod.rs              # Public API
├── simplify.rs         # Expression simplification
├── expand.rs           # Expression expansion
├── factor.rs           # Polynomial factorization
├── derivative.rs       # Already exists — extend
├── integrate.rs        # Symbolic integration (Risch-lite)
├── pattern.rs          # Pattern matching engine
└── rules.rs            # Rewrite rules database
```

### Features & Priority

| Feature | Complexity | Dependencies |
|---------|------------|--------------|
| **Simplify** (constant folding, `x+0=x`, `x*1=x`, `x^1=x`) | Low | None |
| **Expand** (`(a+b)^n`, `a*(b+c)`) | Low | None |
| **Collect** (`x^2 + 2x + x^2` → `2x^2 + 2x`) | Medium | Expand |
| **Factor** (quadratic, cubic, difference of squares, grouping) | Medium | Collect |
| **Partial Fractions** | Medium | Factor |
| **Symbolic Derivative** | Done | — |
| **Symbolic Integral** (polynomials, rational, basic trig) | High | Factor, Partial Fractions |
| **Pattern Matcher** (unification, substitution) | Medium | Core for all above |

### CLI Commands (Phase 2)
```bash
> simplify sin(x)^2 + cos(x)^2
  1

> expand (x+1)^5
  x^5 + 5*x^4 + 10*x^3 + 10*x^2 + 5*x + 1

> factor x^4 - 1
  (x-1)*(x+1)*(x^2+1)

> derivative sin(x^2), x
  2*x*cos(x^2)

> integrate x^2, x
  x^3/3
```

### Testing Strategy
- Golden-file tests: `tests/symbolic/*.in` → `*.out`
- Property tests: `simplify(expand(expr)) == expr` (modulo commutativity)
- Benchmark: simplify vs Mathematica/SymPy on standard suite

---

## Phase 3: Interactive TUI Dashboard (Week 4-5)
**Goal**: Rich terminal UI with live plot, history, multi-pane layout.

### Tech Stack
- **ratatui** (tui-rs fork) — mature, actively maintained
- **crossterm** — cross-platform terminal control
- **plotters** — already in deps, supports `ratatui` backend via `plotters-crossterm`

### UI Layout
```
┌────────────────────────────────────────────────────────────┐
│ SMS v0.2  │  Equation: sin(x) = 0.5        │  Mode: Adaptive │
├──────────┼──────────────────────────────────┼─────────────────┤
│ History  │ Solutions                      │ Plot (live)     │
│ ──────── │ ────────────────────────────── │ ─────────────── │
│ 1.x^2-4  │ 1. x = 0.523599  (err: 1e-15)  │ ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁ │
│ 2.sin(x) │ 2. x = 2.618    (err: 2e-15)   │ █▇▆▅▄▃▂▁▂▃▄▅▆▇█ │
│ 3.x^3-2  │ 3. x = 1.26     (err: 5e-10)   │ ────────────── │
│ ▼        │                                 │ -π    0    π    │
│          │ [Tab] Switch pane  [Enter] Edit │                 │
├──────────┴──────────────────────────────────┴─────────────────┤
│ > sin(x) = 0.5                                        [Solve] │
└────────────────────────────────────────────────────────────┘
```

### Components
| Component | File | Responsibility |
|-----------|------|----------------|
| `App` | `tui/app.rs` | State machine, event loop |
| `InputPane` | `tui/input.rs` | Equation editor with syntax highlight |
| `SolutionPane` | `tui/solutions.rs` | Scrollable list, copy/export actions |
| `PlotPane` | `tui/plot.rs` | Live plotters canvas, zoom/pan |
| `HistoryPane` | `tui/history.rs` | Searchable, persistent (SQLite) |
| `Keymap` | `tui/keys.rs` | Vim/Emacs bindings, help overlay |

### Key Features
- **Live solving**: Debounced (300ms) solve on keystroke
- **Plot sync**: Click solution → highlight on plot; hover plot → show x,y
- **History persistence**: SQLite (`data/history.db`), survives restarts
- **Export from TUI**: `e` → JSON/CSV/LaTeX picker
- **Config file**: `~/.config/sms/config.toml` (theme, keybindings, solver defaults)

### Build
```toml
# sms-tui/Cargo.toml
[dependencies]
ratatui = "0.28"
crossterm = "0.28"
plotters = { version = "0.3", features = ["crossterm"] }
sms-core = { path = "../sms-core" }
rusqlite = "0.31"  # history
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

---

## Phase 4: Unit & Dimension Analysis (Week 5-6)
**Goal**: Track physical units through all calculations.

### Architecture
```
sms-core/src/units/
├── mod.rs              # Public API
├── dimension.rs        # Base dimensions (L, M, T, Q, Θ, N, J)
├── unit.rs             # Unit definitions (m, kg, s, N, J, Pa, etc.)
├── quantity.rs         # Value + Unit (with ops)
├── registry.rs         # Unit registry (SI + custom)
├── parser.rs           # Parse "5 m/s^2" → Quantity
├── formatter.rs        # Format with best prefix (k, M, µ, n)
└── conversions.rs      # Unit conversion graph
```

### Dimension System (7 SI base)
| Dimension | Symbol | Base Unit |
|-----------|--------|-----------|
| Length | L | meter (m) |
| Mass | M | kilogram (kg) |
| Time | T | second (s) |
| Current | I | ampere (A) |
| Temperature | Θ | kelvin (K) |
| Amount | N | mole (mol) |
| Luminous | J | candela (cd) |

### Derived Units (partial)
| Unit | Expression | Dimension |
|------|------------|-----------|
| Newton (N) | kg·m/s² | M·L·T⁻² |
| Joule (J) | N·m | M·L²·T⁻² |
| Watt (W) | J/s | M·L²·T⁻³ |
| Pascal (Pa) | N/m² | M·L⁻¹·T⁻² |
| Coulomb (C) | A·s | I·T |
| Volt (V) | J/C | M·L²·T⁻³·I⁻¹ |
| Ohm (Ω) | V/A | M·L²·T⁻³·I⁻² |

### Integration Points
| Module | Change |
|--------|--------|
| `expr.rs` | `Expr::Quantity(Quantity)` variant |
| `parser.rs` | Parse `5 m/s^2`, `9.81 m/s^2 * 2 kg` |
| `solver.rs` | Dimensional consistency check before solve |
| `series.rs` | Unit-aware summation |
| `export.rs` | Export with units |

### CLI Examples
```bash
> 5 m/s * 10 s
  50 m

> 9.81 m/s^2 * 2 kg
  19.62 N

> sqrt(2 * 9.81 m/s^2 * 10 m)
  14 m/s

> E = m*c^2, m=1kg, c=299792458 m/s
  E = 8.98755e16 J

> convert 100 km/h to m/s
  27.7778 m/s

> dimension of G
  M⁻¹·L³·T⁻²
```

### Testing
- Dimensional correctness: `add(m, s)` → error
- Conversion round-trips: `m → km → m`
- Prefix formatting: `0.000001 m` → `1 µm`

---

## Phase 5: WASM + Web Playground (Week 6-7)
**Goal**: Self-contained HTML playground, zero-install demo.

### Architecture
```
sms-wasm/
├── Cargo.toml
├── src/
│   ├── lib.rs          # wasm-bindgen exports
│   ├── wasm.rs         # JS glue
│   └── panic_hook.rs   # Better error messages
├── www/                # Generated by wasm-pack
│   ├── index.html
│   ├── sms.js
│   ├── sms_bg.wasm
│   └── style.css
└── playground/         # Source for index.html
    ├── index.html
    ├── style.css
    └── app.js          # Vanilla JS, no framework
```

### wasm-bindgen Exports (`sms-wasm/src/lib.rs`)
```rust
#[wasm_bindgen]
pub struct WasmSolver {
    solver: EquationSolver,
}

#[wasm_bindgen]
impl WasmSolver {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { ... }

    pub fn solve(&self, equation: &str) -> JsValue { ... }
    pub fn solve_advanced(&self, equation: &str, mode: &str) -> JsValue { ... }
    pub fn simplify(&self, expr: &str) -> JsValue { ... }
    pub fn derivative(&self, expr: &str, var: &str) -> JsValue { ... }
    pub fn plot_data(&self, expr: &str, var: &str, min: f64, max: f64, n: usize) -> JsValue { ... }
    pub fn export_json(&self, data: &str) -> JsValue { ... }
}
```

### Playground Features (vanilla JS, ~200 lines)
- **Editor**: CodeMirror 6 (lightweight) or `<textarea>` + Prism.js highlight
- **Plot**: Chart.js or Canvas API (plot_data returns `[[x,y],...]`)
- **History**: localStorage
- **Export**: Download buttons (JSON/CSV/LaTeX)
- **Share**: URL hash encodes equation (`#eq=sin(x)=0.5`)

### Build & Deploy
```bash
# Build
cd sms-wasm
wasm-pack build --target web --out-dir www

# Serve locally
npx serve www

# Deploy to GitHub Pages / Netlify / Cloudflare Pages
# → https://zaidejjo.github.io/sms/
```

### Size Budget
| Asset | Target |
|-------|--------|
| `sms_bg.wasm` (gzip) | < 300 KB |
| `sms.js` (gzip) | < 20 KB |
| Total first load | < 400 KB |

---

## Phase 6: Polish & Release (Week 7-8)

### Cross-Feature Integration
- [ ] TUI uses symbolic engine (simplify button)
- [ ] TUI uses units (unit-aware solve)
- [ ] WASM exposes symbolic + units APIs
- [ ] CLI `sms` binary includes all features (feature flags)

### Documentation
- [ ] `docs/symbolic.md` — rewrite rules, examples
- [ ] `docs/tui.md` — keybindings, config, screenshots
- [ ] `docs/units.md` — unit registry, defining custom units
- [ ] `docs/wasm.md` — embedding in web pages
- [ ] Update `README.md` with all 4 features

### Packaging
- [ ] AUR package update (`sms`, `sms-tui`, `sms-wasm` split or meta-package)
- [ ] Homebrew formula
- [ ] Windows Scoop/Chocolatey
- [ ] GitHub Actions: build + test + release artifacts

---

## Dependency Graph

```
Phase 1 (Foundation)
    │
    ├──→ Phase 2 (Symbolic) ──────┐
    │                             │
    ├──→ Phase 3 (TUI) ◄──────────┤  (uses Symbolic)
    │                             │
    ├──→ Phase 4 (Units) ◄────────┤  (uses Symbolic for unit expressions)
    │                             │
    └──→ Phase 5 (WASM) ◄─────────┘  (uses Symbolic, Units)
            │
            └──→ Phase 6 (Polish)
```

---

## Effort Summary

| Phase | Weeks | Lines of Code (est.) | Risk |
|-------|-------|---------------------|------|
| 1. Foundation | 1-2 | ~500 | Low |
| 2. Symbolic | 2-3 | ~2500 | Medium (factorization) |
| 3. TUI | 1-2 | ~1500 | Low (ratatui is stable) |
| 4. Units | 1-2 | ~1200 | Medium (dimension inference) |
| 5. WASM | 1 | ~800 | Low (wasm-bindgen mature) |
| 6. Polish | 1 | ~500 | Low |
| **Total** | **7-10** | **~7000** | — |

---

## Feature Flags (Cargo.toml)

```toml
[features]
default = ["cli"]
cli = ["sms-core"]
tui = ["sms-core", "sms-tui"]
wasm = ["sms-core", "sms-wasm"]
symbolic = ["sms-core/symbolic"]
units = ["sms-core/units"]
all = ["cli", "tui", "wasm", "symbolic", "units"]
```

---

## Quick Start Commands

```bash
# Phase 1: Split workspace
cargo new --lib sms-core
cargo new --bin sms-cli

# Phase 2: Symbolic
# Edit sms-core/src/symbolic/...

# Phase 3: TUI
cargo new --bin sms-tui
# Add ratatui, crossterm, plotters-crossterm

# Phase 4: Units
# Edit sms-core/src/units/...

# Phase 5: WASM
cargo new --lib sms-wasm
# Add wasm-bindgen, wasm-pack

# Build all
cargo build --workspace --release
cargo build --workspace --release --features tui
cd sms-wasm && wasm-pack build --target web
```

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Symbolic test coverage | > 90% |
| TUI keystroke-to-solve latency | < 100ms |
| Unit conversion accuracy | Exact (rational) where possible |
| WASM load time (3G) | < 3s |
| Binary size (release, stripped) | < 3 MB |
| AUR/package install time | < 30s |