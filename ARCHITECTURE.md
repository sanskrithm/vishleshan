# Pāṇini-RS: Detailed Architecture

## System Design Overview

Pāṇini-RS is a multi-stage compiler that treats Sanskrit morphological structure as semantic executable code. This document provides a deep dive into the architecture.

## 1. LEXER (token.rs + lexer.rs)

### Lexical Tokens

```rust
pub enum Token {
    Morpheme(MorphemeCompound),  // root-case pair
    Verb(VerbForm),              // dhātu-execution pair
    Eof,
}

pub struct MorphemeCompound {
    root: String,                // identifier
    case: CaseMarker,            // grammatical case
}

pub struct VerbForm {
    dhatu: Dhatu,                // operation root
    execution: ExecutionMarker,  // control flow
}
```

### Lexer Implementation

**File**: `src/lexer.rs`
**Algorithm**: Finite state machine without regex

```
Input: "data-āt sales-ena chid-tvā"

Split on whitespace:
  ["data-āt", "sales-ena", "chid-tvā"]

For each token:
  1. Split on '-' → (root, suffix)
  2. Parse suffix:
     - If case marker (-āt, -ena):
       Create Token::Morpheme
     - If execution marker (-tvā, -ti):
       Parse dhātu(root)
       Create Token::Verb

Output: [Morpheme(data, āt), Morpheme(sales, ena), Verb(chid, Continue), EOF]
```

**Key Features**:
- Hand-written (no regex) for predictability
- UTF-8 aware for Sanskrit characters
- Error recovery with diagnostic messages
- Suffix matching via simple string comparison

**Time Complexity**: O(n) where n = input length
**Space Complexity**: O(m) where m = token count

## 2. PARSER (parser.rs)

### Parsing Strategy

**Algorithm**: Recursive descent with 3 parsing phases

```rust
pub fn parse(&mut self) -> ParseResult<Program> {
    // Phase 1: Extract source (Apādāna, -āt)
    program.source = self.parse_source()?;
    
    // Phase 2: Extract instruments (Karaṇa, -ena)
    program.instruments = self.parse_instruments()?;
    
    // Phase 3: Extract operations (verbal morphemes)
    program.operations = self.parse_operations()?;
    
    // Phase 4: Validate structure
    program.validate()?;
    
    Ok(program)
}
```

### Phase 1: Source Extraction

```
Expect: First token is Morpheme with CaseMarker::Source

Input tokens: [Morpheme(data, āt), ...]

peek() → Morpheme(data, āt)
  ✓ Case is Source?
    → OK, extract "data"
    → advance()
    → return "data"
```

**Constraint**: Must be first and unique

### Phase 2: Instrument Extraction

```
Loop while tokens are Morpheme with CaseMarker::Instrument

Input: [Morpheme(sales, ena), Morpheme(region, ena), Verb(...), ...]

Loop iteration 1:
  peek() → Morpheme(sales, ena)
    → case == Instrument? ✓
    → append "sales"
    → advance()

Loop iteration 2:
  peek() → Morpheme(region, ena)
    → case == Instrument? ✓
    → append "region"
    → advance()

Loop iteration 3:
  peek() → Verb(...)
    → case == Instrument? ✗
    → break (operations begin)

Return: ["sales", "region"]
```

**Anuvṛtti Semantics**: All instruments become available to all operations

### Phase 3: Operation Extraction

```
Loop while tokens are Verb

Input: [Verb(chid, tvā), Verb(yuj, tvā), Verb(dṛś, ti), EOF]

Loop iteration 1:
  peek() → Verb(chid, tvā)
    → create Operation(chid, Continue)
    → advance()
    → execution != Terminal? Continue loop

Loop iteration 2:
  peek() → Verb(yuj, tvā)
    → create Operation(yuj, Continue)
    → advance()
    → execution != Terminal? Continue loop

Loop iteration 3:
  peek() → Verb(dṛś, ti)
    → create Operation(dṛś, Terminal)
    → advance()
    → execution == Terminal? ✓ Break

Return: [Operation(chid, Continue), Operation(yuj, Continue), Operation(dṛś, Terminal)]
```

**Constraint**: Final operation MUST be terminal (-ti)

### Validation

```rust
fn validate(&self) -> Result<(), String> {
    ✓ source is non-empty
    ✓ operations list is non-empty
    ✓ final operation is terminal
}
```

## 3. ABSTRACT SYNTAX TREE (ast.rs)

### AST Structure

```rust
pub struct Program {
    pub source: String,                // Source dataset (Apādāna)
    pub instruments: Vec<String>,      // Inherited parameters (Karaṇa)
    pub operations: Vec<Operation>,    // Execution pipeline
}

pub struct Operation {
    pub dhatu: Dhatu,                  // Semantic operation
    pub execution: ExecutionMarker,    // Lazy vs terminal
}
```

### AST Example

For input: `data-āt sales-ena chid-tvā yuj-tvā dṛś-ti`

```rust
Program {
    source: "data",
    instruments: vec!["sales"],
    operations: vec![
        Operation {
            dhatu: Dhatu::Chid,
            execution: ExecutionMarker::Continue,
        },
        Operation {
            dhatu: Dhatu::Yuj,
            execution: ExecutionMarker::Continue,
        },
        Operation {
            dhatu: Dhatu::Drsh,
            execution: ExecutionMarker::Terminal,
        },
    ],
}
```

## 4. SEMANTIC ANALYZER & QUERY PLANNER (compiler.rs)

### CompilerState: Anuvṛtti Implementation

```rust
pub struct CompilerState {
    context: HashSet<String>,    // Inherited parameters (Anuvṛtti)
    steps: Vec<QueryStep>,       // Compiled query steps
}
```

### Query Planning Algorithm

```
compile(program: &Program) → QueryPlan

1. Initialize:
   - context = {}
   - steps = []

2. Add source loading:
   steps.push(LoadSource(program.source))

3. Bind instruments (Anuvṛtti):
   for instrument in program.instruments:
       context.insert(instrument)

4. Compile operations:
   for operation in program.operations:
       steps.push(compile_operation(operation))

5. Return QueryPlan { steps, context }
```

### Operation Compilation

#### chid (Filter)
```
Compile chid-tvā using inherited context:

operation: Operation(Chid, Continue)
context: {"sales", "region"}

→ QueryStep::Filter {
    conditions: ["sales IS NOT NULL", "region IS NOT NULL"]
  }
```

#### ci (Group-By)
```
Compile ci-tvā using inherited context:

operation: Operation(Ci, Continue)
context: {"region", "sales"}

→ QueryStep::GroupBy {
    columns: ["region", "sales"]
  }
```

#### yuj (Aggregate)
```
Compile yuj-tvā using inherited context:

operation: Operation(Yuj, Continue)
context: {"sales"}

→ QueryStep::Aggregate {
    operation: "sum",
    columns: ["sales"]
  }
```

#### dṛś (Render)
```
Compile dṛś-ti (terminal):

operation: Operation(Drsh, Terminal)

→ QueryStep::Render
   (triggers pipeline execution)
```

### Query Plan Example

Input: `data-āt sales-ena chid-tvā yuj-tvā dṛś-ti`

```rust
QueryPlan {
    context: { "sales" },
    steps: [
        LoadSource { dataset: "data" },
        Filter { conditions: ["sales IS NOT NULL"] },
        Aggregate { operation: "sum", columns: ["sales"] },
        Render,
    ]
}
```

## 5. LOWERING TO POLARS

### LazyFrame Generation

Each QueryStep maps to Polars methods:

```
QueryStep::LoadSource("data")
  → LazyFrame = df("data").lazy()

QueryStep::Filter { ... }
  → lazy_frame.filter(col("sales").gt(lit(0)))

QueryStep::GroupBy { columns: ["region"] }
  → lazy_frame.groupby(vec![col("region")])

QueryStep::Aggregate { op: "sum", cols: ["sales"] }
  → lazy_frame.agg(vec![col("sales").sum()])

QueryStep::Render
  → lazy_frame.collect()
  → println!("{:?}", dataframe)
```

## 6. ERROR HANDLING

### Error Types

```
LexError
├── MissingSeparator { input }
├── UnknownCaseMarker { suffix }
├── UnknownExecutionMarker { suffix }
├── UnknownDhatu { root }
└── InvalidUtf8 { details }

ParseError
├── UnexpectedEof
├── NoSourceMorpheme
├── ExpectedMorpheme { got }
├── ExpectedVerb { got }
├── InstrumentsAfterOperations
├── MultipleSourceDeclations
├── NoTerminalOperation
└── ValidationError { reason }

CompilerError
├── UnknownDhatu { dhatu }
├── OperationRequiresContext { dhatu }
└── QueryPlanError { reason }
```

### Error Propagation

```
main()
  ├─→ Lexer::tokenize() → LexError
  ├─→ Parser::parse() → ParseError
  └─→ CompilerState::compile() → CompilerError

Each layer enriches error with context:
✗ "sales-ena dṛś-ti"
  ↑ "No source morpheme (missing -āt suffix)"
```

## 7. TYPE SYSTEM

### Key Types

```rust
// Morphological units
enum CaseMarker { Source, Instrument }
enum ExecutionMarker { Continue, Terminal }
enum Dhatu { Chid, Ci, Yuj, Drsh, Custom(String) }

// Compound units
struct MorphemeCompound { root, case }
struct VerbForm { dhatu, execution }

// Tokens
enum Token { Morpheme, Verb, Eof }

// AST
struct Operation { dhatu, execution }
struct Program { source, instruments, operations }

// Compiled form
enum QueryStep { LoadSource, Filter, GroupBy, Aggregate, Render }
struct QueryPlan { steps, context }
```

## 8. MEMORY LAYOUT

### Token Storage (Stack)
```
Token::Morpheme
├── root: String (heap-allocated)
└── case: CaseMarker (enum, small)

Token::Verb
├── dhatu: Dhatu (enum)
└── execution: ExecutionMarker (enum, small)
```

### Program Storage (Heap)
```
Program
├── source: String (heap)
├── instruments: Vec<String> (heap)
└── operations: Vec<Operation>
    └── [Operation, Operation, ...]
        ├── dhatu: Dhatu
        └── execution: ExecutionMarker
```

### Context Storage (Hash Table)
```
HashSet<String>
├── "sales"
├── "region"
└── "discount"
```

## 9. TIME & SPACE COMPLEXITY

### Lexical Analysis
- **Time**: O(n) where n = input length
- **Space**: O(m) where m = token count

### Parsing
- **Time**: O(m) where m = token count
- **Space**: O(1) auxiliary (AST allocated separately)

### Semantic Analysis
- **Time**: O(k) where k = operation count
- **Space**: O(k + c) where c = context size

### Overall Complexity
- **Time**: O(n) linear in input
- **Space**: O(n) in worst case (large dataset names)

## 10. DESIGN PATTERNS

### Pattern 1: Recursive Descent Parser
```rust
fn parse(&mut self) -> ParseResult<Program> {
    source = self.parse_source()?;
    instruments = self.parse_instruments()?;
    operations = self.parse_operations()?;
    Ok(Program { source, instruments, operations })
}
```

### Pattern 2: Strongly Typed Tokens
```rust
enum Token {
    Morpheme(MorphemeCompound),
    Verb(VerbForm),
    Eof,
}
```
Exhaustive pattern matching prevents bugs.

### Pattern 3: Anuvṛtti as HashSet
```rust
context: HashSet<String>  // Once bound, inherited by all
```
O(1) membership check, automatic deduplication.

### Pattern 4: QueryPlan as Intermediate Form
```rust
AST → QueryPlan → [Future backends]
```
Allows multiple backends without changing parser/lexer.

## 11. VALIDATION & VERIFICATION

### Structural Validation
```
✓ Source declaration present
✓ Source is unique
✓ Instruments appear before operations
✓ Final operation is terminal
```

### Semantic Validation
```
✓ All referenced operations exist
✓ Operations have required context
✓ Terminal operation is terminal
```

### Testing Strategy
```
Unit Tests (each module)
├── token: display, equality
├── lexer: tokenization, error cases
├── parser: phases, validation
├── ast: structure, traversal
└── compiler: planning, context

Integration Tests (full pipeline)
├── Simple programs (load + render)
├── With instruments (Anuvṛtti)
├── Complex pipelines (multiple ops)
└── Error cases (missing source, etc.)
```

## 12. FUTURE EXTENSIONS

### 1. LLVM Backend
```
QueryPlan → LLVM IR
└─ @load_source(i8*)
└─ @filter(%LazyFrame*, i8*)
└─ @groupby(%LazyFrame*, i8*)
└─ @sum(%LazyFrame*, i8*)
└─ @print_df(%LazyFrame*)
```

### 2. SQL Backend
```
QueryPlan → SQL
└─ SELECT * FROM data
   WHERE sales > 0
   GROUP BY region
   AGGREGATE sum(sales)
```

### 3. Type System
```
Nominal type system with:
├── Column types (int, string, float)
├── Type checking at compile time
└── Generic operations
```

### 4. Macro System
```
Syntax: operation-name{
  argument1,
  argument2
}

Expands to multiple operations
```

---

**Pāṇini-RS: Where Sanskrit Grammar Meets Modern Compilation**
