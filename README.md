# Pāṇini-RS: A Morphology-Driven Sanskrit DSL for Data Processing

A production-grade compiler for **Pāṇini-RS**, a Sanskrit-inspired morphological programming language built in Rust. This is a sophisticated compiler that treats Sanskrit grammatical morphology as executable semantic code.

## 🏗️ Architecture Overview

The compiler implements a complete, classical compiler pipeline:

```
Input DSL (Sanskrit Morphemes)
    ↓
Lexer (Lexical Analysis) → Token Stream
    ↓
Parser (Syntactic Analysis) → Abstract Syntax Tree (AST)
    ↓
Semantic Analyzer (Compiler State) → Query Plan
    ↓
Runtime Execution → Polars LazyFrame → Output
```

## 🎯 Core Philosophy

**This is NOT Sanskrit syntax layered on SQL.**

Pāṇini-RS treats Sanskrit grammatical morphology as **executable compiler semantics**:

- **Suffixes define relationships**: Not traditional positional arguments
- **Cases encode semantics**: Grammatical cases (Kārakas) map to semantic operations
- **Non-positional dispatch**: Operations are routed through case markers, not position
- **State inheritance**: Anuvṛtti (automatic inheritance) flows through the pipeline

## 📚 Language Specification

### Kāraka Suffixes (Grammatical Cases as Semantic Relations)

Kārakas are Sanskrit grammatical cases that encode semantic relationships:

#### Source/Apādāna: `-āt`
- **Purpose**: Declares the source dataset
- **Position**: Must be first morpheme
- **Compiler Meaning**: Bind source data
- **Example**: `data-āt` loads the dataset "data"

#### Instrument/Karaṇa: `-ena`
- **Purpose**: Declares inherited parameters
- **Compiler Meaning**: Bind parameter to execution context (Anuvṛtti)
- **Example**: `sales-ena` makes "sales" available to all operations
- **Anuvṛtti**: Once declared, automatically inherited by subsequent operations

### Sūtra Suffixes (Control Flow)

Sūtras encode execution semantics—whether operations are lazy or terminal:

#### Pipeline Continuation: `-tvā`
- **Purpose**: Lazy operation (continue pipeline)
- **Compiler Meaning**: Add to LazyFrame, don't execute yet
- **Example**: `chid-tvā` applies filter but defers execution

#### Terminal Execution: `-ti`
- **Purpose**: Terminal operation (execute pipeline)
- **Compiler Meaning**: Trigger pipeline evaluation and output
- **Example**: `dṛś-ti` renders final result
- **Constraint**: Must be the final operation

### Dhātu Roots (Semantic Operations)

Dhātus are verb roots that encode semantic operations. Inspired by the Dhātupāṭha (Sanskrit verb root catalog):

| Dhātu | Meaning | Compiler Behavior | Polars Lowering |
|-------|---------|-------------------|-----------------|
| `chid` | Cut/Filter | Apply predicate filtering | `.filter()` |
| `ci` | Gather/Group | Group-by aggregation | `.group_by()` |
| `yuj` | Join/Aggregate | Numerical aggregation | `.agg()` with sum |
| `dṛś` / `drsh` | See/Render | Execute pipeline and output | `.collect()` + print |

## 📖 Example Program

### Input
```
data-āt sales-ena chid-tvā yuj-tvā dṛś-ti
```

### Semantic Interpretation

Breaking down morpheme by morpheme:

1. **`data-āt`** (Source, Apādāna)
   - Load dataset "data"
   - Establishes the source for the pipeline

2. **`sales-ena`** (Instrument, Karaṇa)
   - Bind "sales" as inherited parameter
   - Available to all subsequent operations (Anuvṛtti)

3. **`chid-tvā`** (Filter, Lazy)
   - Filter operation using "sales"
   - Lazy: deferred execution

4. **`yuj-tvā`** (Aggregate, Lazy)
   - Sum aggregation on "sales"
   - Lazy: still deferred

5. **`dṛś-ti`** (Render, Terminal)
   - Render/visualize output
   - Terminal: triggers pipeline execution

### Compiled Query Plan

```
Program {
  source: "data",
  instruments: ["sales"],
  operations: [
    Operation(Chid, Continue),    // Filter using "sales"
    Operation(Yuj, Continue),     // Aggregate using "sales"
    Operation(Drsh, Terminal),    // Render output
  ]
}

↓ (Compilation)

QueryPlan {
  context: { "sales" },           // Anuvṛtti context
  steps: [
    LoadSource("data"),           // Phase 1: Load
    Filter(["sales IS NOT NULL"]), // Phase 2: Filter (lazy)
    Aggregate(sum, ["sales"]),    // Phase 3: Aggregate (lazy)
    Render                        // Phase 4: Render (terminal)
  ]
}
```

## 🏗️ Project Structure

```
src/
├── dhatu.rs          # Dhatu/opcode definitions (aggregate/transform/relational)
├── graph.rs          # Semantic Graph IR (nodes, Karaka edges, propagation)
├── lexer.rs          # ASCII-only tokenizer (FSM)
├── semantic.rs       # Semantic analyzer (Anuvrtti + graph builder)
├── optimizer.rs      # Paribhasha rewrite engine (query optimizer)
├── planner.rs        # Execution planner (ExecutionPlan IR)
├── lib.rs            # Library exports
└── main.rs           # CLI/REPL and examples

tests/
└── integration_tests.rs  # End-to-end compilation & planning tests

.github/
└── workflows/        # CI (runs cargo fmt/clippy/tests)

docs/
├── README.md         # This file
├── SEMANTIC_GRAPH_ARCHITECTURE.md   # Detailed architecture
└── MIGRATION_v0.1_TO_v0.2.md      # Migration guide

Cargo.toml           # Rust package manifest
LICENSE               # Apache-2.0 license (renamed from LICENSE.tcl)
.gitignore            # Excludes /target/ and editor files
```

## 🔧 Module Descriptions

### `token.rs` - Lexical Units
Defines strongly-typed tokens:
- `CaseMarker`: Grammatical cases (-āt, -ena)
- `ExecutionMarker`: Control flow (-tvā, -ti)
- `Dhatu`: Semantic operations (chid, ci, yuj, dṛś)
- `MorphemeCompound`: Nominal forms (root + case)
- `VerbForm`: Verbal forms (dhātu + execution)

**Lines**: 200 | **Key Concepts**: Pattern matching, UTF-8 handling

### `lexer.rs` - Tokenization
Hand-written lexer (no regex):
- Splits input on whitespace
- Parses morpheme structure (root-suffix)
- Validates case and execution markers
- Handles UTF-8 Sanskrit characters
- Error recovery with diagnostic messages

**Lines**: 400 | **Algorithm**: Finite state machine, suffix matching

### `parser.rs` - Syntactic Analysis
Recursive descent parser (legacy placeholder): the current pipeline uses `lexer.rs` + `semantic.rs` directly to build the SemanticGraph. Parser module is available as a future layering option.

**Status**: parsing responsibilities are implemented in `semantic.rs` (three-phase analyzer)

### `ast.rs` - Abstract Syntax Tree
AST representations:
- `Operation`: Single pipeline step (dhātu + execution marker)
- `Program`: Complete program structure
  - `source`: Dataset name
  - `instruments`: Inherited parameters (Anuvṛtti)
  - `operations`: Execution pipeline

**Lines**: 300 | **Concepts**: Validation, introspection methods

### `semantic.rs` - Semantic Analysis
Three-phase analyzer (source → instruments → operations) that directly constructs the `SemanticGraph` IR, performs Karaka resolution and Anuvrtti propagation, and validates pipeline constraints. This is where parsing and semantic checks are centralized.

**Lines**: 350 | **Semantics**: Karaka resolution, Anuvrtti, graph construction

### `optimizer.rs` - Query Optimization (Paribhasha)
Implements a Paribhasha rewrite engine with modular passes for redundancy elimination, filter pushdown, aggregation coalescing, and projection pruning. Provides `QueryOptimizer` and `OptimizationPass` trait.

**Status**: Implemented (basic passes scaffolding) | **Location**: `src/optimizer.rs`

### `planner.rs` - Execution Planning
Converts `SemanticGraph` → `ExecutionPlan` (steps) with backend selection hooks (Polars, Arrow, DuckDB, DataFusion). Emits `ExecutionStep` IR suitable for backend lowering.

**Status**: Implemented (ExecutionPlan IR + planner) | **Location**: `src/planner.rs`

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+ (2021 edition)
- Cargo
- Polars 0.35+

### Build
```bash
cargo build --release
```

### Run
```bash
# Interactive REPL
cargo run -- repl

# Run example programs
cargo run -- examples

# Compile a specific program
cargo run -- "data-āt sales-ena chid-tvā dṛś-ti"

# Get help
cargo run -- --help
```

### Testing
```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_full_pipeline_example
```

## 📊 Example Programs

### 1. Simple Load and Render
```
data-āt dṛś-ti
```
Loads dataset "data" and renders it.

### 2. Load with Filter
```
data-āt sales-ena chid-tvā dṛś-ti
```
Loads "data", filters by "sales", renders result.

### 3. Filter and Aggregate
```
data-āt sales-ena chid-tvā yuj-tvā dṛś-ti
```
Load → Filter by sales → Sum sales → Render

### 4. Multiple Instruments
```
data-āt region-ena sales-ena ci-tvā dṛś-ti
```
Load → Group by region and sales → Render

### 5. Complex Pipeline
```
data-āt region-ena sales-ena chid-tvā ci-tvā yuj-tvā dṛś-ti
```
Load → Filter → Group by region and sales → Sum → Render

## 🔬 Compiler Implementation Details

### Lexical Analysis
- **Input**: Raw DSL string
- **Output**: Token stream
- **Algorithm**: Whitespace splitting + suffix matching
- **Key Feature**: No regex; hand-written scanner

### Parsing
- **Strategy**: Recursive descent
- **Pattern Matching**: Exhaustive match on token types
- **Error Recovery**: Structured error types with context
- **Validation**: Structural constraints (source first, terminal last)

### Semantic Analysis
- **Anuvṛtti Implementation**: HashSet-based context
- **Query Planning**: AST → QueryPlan transformation
- **Lowering**: Mapping dhātu to Polars operations
- **Type System**: Strongly typed at every phase

### Key Design Decisions

1. **No Regex in Lexer**: Explicit state machines are more predictable and testable
2. **Strongly Typed Tokens**: Enum-based tokens prevent many classes of bugs
3. **Separate Parsing Phases**: Source → Instruments → Operations enforces structure
4. **Anuvṛtti as HashSet**: Efficient context inheritance
5. **QueryPlan as Intermediate**: Allows future backends (LLVM, SQL, etc.)

## 🧪 Testing Strategy

The test suite covers:

1. **Unit Tests**: Each module has comprehensive unit tests
2. **Integration Tests**: Full pipeline from input to QueryPlan
3. **Error Handling**: Parser and compiler error cases
4. **Edge Cases**: UTF-8 handling, empty tokens, multiple declarations
5. **Example Programs**: All documented examples compile correctly

### Running Tests
```bash
# All tests
cargo test

# Module tests only
cargo test --lib

# Integration tests
cargo test --test integration_tests

# Verbose output
cargo test -- --nocapture --test-threads=1
```

## 🎓 Language Design Philosophy

### Why Sanskrit Morphology?

1. **Compositionality**: Morphemes compose predictably
2. **Case Systems**: Grammatical cases naturally encode semantic relations
3. **Inheritance (Anuvṛtti)**: Sanskrit grammar has a principle of inherited context
4. **Verb Roots**: Dhātus as semantic operations are well-studied in linguistic theory
5. **Non-positional**: Cases eliminate positional ambiguity

### Design Principles

1. **Morphological Semantics**: Meaning lives in morphology, not position
2. **Explicit Case Marking**: All relationships declared via cases
3. **Lazy Evaluation**: -tvā suffix defers execution until terminal operation
4. **Context Inheritance**: Once bound, parameters flow through pipeline
5. **Strong Typing**: Every token is strongly typed; exhaustive pattern matching

## 📋 Rust Engineering

All code follows:
- ✅ Rust 2021 idioms
- ✅ Zero-cost abstractions
- ✅ Memory safety (no unsafe outside FFI)
- ✅ Exhaustive pattern matching
- ✅ No unwrap() in library code
- ✅ Idiomatic error handling
- ✅ Minimal allocations
- ✅ Borrowed slices where possible
- ✅ Immutable structures
- ✅ Modular design

## 🔮 Future Extensions

Potential enhancements:

1. **LLVM IR Backend**: Emit LLVM IR for JIT compilation
2. **SQL Backend**: Lower to SQL for database execution
3. **More Dhātus**: Extend operation vocabulary
4. **Custom Operators**: User-defined semantic operations
5. **Optimization Passes**: Query optimization (predicate pushdown, etc.)
6. **Type System**: Formal type checking
7. **Macro System**: Meta-programming capabilities

## 📞 Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│          Pāṇini-RS Compiler Pipeline                    │
└─────────────────────────────────────────────────────────┘

Input: "data-āt sales-ena chid-tvā yuj-tvā dṛś-ti"
   │
   ├─→ Lexer.tokenize()
   │   Tokens: [Morpheme(data, āt), Morpheme(sales, ena), 
   │             Verb(chid, tvā), Verb(yuj, tvā), Verb(dṛś, ti), EOF]
   │
   ├─→ Parser.parse()
   │   Program {
   │     source: "data",
   │     instruments: ["sales"],
   │     operations: [Operation(chid, tvā), Operation(yuj, tvā), Operation(dṛś, ti)]
   │   }
   │
   ├─→ CompilerState.compile()
   │   QueryPlan {
   │     context: {"sales"},
   │     steps: [LoadSource("data"), Filter(...), Aggregate(...), Render]
   │   }
   │
   └─→ Runtime Execution
       Polars LazyFrame → Collect → Output
```

## 📖 References

This compiler is inspired by:
- **Pāṇini's Aṣṭādhyāyī**: Sanskrit grammatical theory
- **LLVM Frontend Design**: Classical compiler architecture
- **Query Optimizer Design**: Polars execution model
- **Formal Semantics**: Denotational and operational semantics

## 📄 License

This project is released under the MIT License.

---

**Built with Rust | Powered by Sanskrit Morphology | Compiled to Polars**

For detailed architecture documentation, see `ARCHITECTURE.md`.
