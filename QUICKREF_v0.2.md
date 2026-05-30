# PANINI-RS v0.2 QUICK REFERENCE

## Command Syntax

```
[source]-at [instrument₁]-ena [instrument₂]-ena ... [operation]-tva ... [operation]-ti
```

### Components

| Component | Type | Required | Example |
|-----------|------|----------|---------|
| Source | `-at` | YES | `sales-at` |
| Instrument | `-ena` | NO | `revenue-ena` |
| Operation | `-tva` | YES (×N) | `chid-tva`, `yuj-tva` |
| Terminal | `-ti` | YES | `drsh-ti` |

---

## Dhatu Reference

### Aggregation (Yāvat Gaṇa)
| Dhatu | ASCII | Meaning | Polars | Example |
|-------|-------|---------|--------|---------|
     ↓ Lexer (src/lexer.rs)
| Madh | `madh` | Average | `.mean()` | `madh-tva` |
| Gan | `gan` | Count | `.count()` | `gan-tva` |
| Lagh | `lagh` | Minimum | `.min()` | `lagh-tva` |
| Mah | `mah` | Maximum | `.max()` | `mah-tva` |

### Transformation (Sādhana Gaṇa)
| Dhatu | ASCII | Meaning | Polars | Example |
|-------|-------|---------|--------|---------|
| Chid | `chid` | Filter | `.filter()` | `chid-tva` |
| Adhyaya | `adhyaya` | Select/Project | `.select()` | `adhyaya-tva` |
| Kram | `kram` | Sort | `.sort()` | `kram-tva` |
| Vibhaj | `vibhaj` | Partition | `.partition_by()` | `vibhaj-tva` |

### Relational (Sandhana Gaṇa)
| Dhatu | ASCII | Meaning | Polars | Example |
|-------|-------|---------|--------|---------|
| Ci | `ci` | Group | `.group_by()` | `ci-tva` |
| Mel | `mel` | Join | `.join()` | `mel-tva` (reserved) |

### Terminal (Nirvartana)
| Dhatu | ASCII | Meaning | Polars | Example |
  lexer.rs        - ASCII tokenizer (no regex)
| Drsh | `drsh` | Render/Execute | `.collect()` | `drsh-ti` |
  main.rs         - CLI/REPL interface
### Predicate (Adhikara Gaṇa) - Reserved v0.3
| Dhatu | ASCII | Meaning | Polars | Example |
|-------|-------|---------|--------|---------|
| Adhik | `adhik` | Greater than | `.gt()` | `adhik-tva` |
| Nyun | `nyun` | Less than | `.lt()` | `nyun-tva` |
| Sam | `sam` | Equal | `.eq()` | `sam-tva` |
| Asam | `asam` | Not equal | `.neq()` | `asam-tva` |

---

## Sūtra Suffixes (Execution Control)
4. **src/main.rs** - How to use the CLI
| Suffix | ASCII | Meaning | Behavior |
|--------|-------|---------|----------|
| -tvā | `-tva` | Pipeline (lazy) | Continue lazy evaluation |
| -ti | `-ti` | Terminal | Collect/execute results |

---
4. **src/main.rs** - How to use the CLI
## Kāraka Relations (Semantic Roles)

| Karaka | Suffix | Meaning | Graph Role |
|--------|--------|---------|------------|
| Apādāna | `-at` | Source/origin | Entry point |
| Karaṇa | `-ena` | Instrument/context | Parameter binding |
| Karma | `-asya` | Object/target | Output binding (v0.3) |
| Adhikarana | (implicit) | Locus/scope | Scope propagation |

---

## Anuvrtti (Context Inheritance)

Once declared with `-ena`, a parameter is automatically inherited by all operations:

```
sales-at revenue-ena chid-tva yuj-tva madh-tva drsh-ti
         ^^^^^^^^^^^
         Declared here once
         ↓↓↓ Automatically inherited ↓↓↓
         [chid uses revenue]
         [yuj uses revenue]
         [madh uses revenue]
```

**No re-declaration needed!**

---

## REPL Commands

```
help          - Show syntax help
examples      - Run 4 example programs
clear         - Clear screen
quit / exit   - Exit REPL
[program]     - Execute Panini program
```

---

## Example Programs

### 1. Filter + Sum
```
sales-at revenue-ena chid-tva yuj-tva drsh-ti
```
Interpretation:
1. Load `sales` dataset
2. Bind `revenue` as filtering parameter
3. Filter by revenue
4. Sum revenue values
5. Render results

### 2. Group + Average
```
data-at region-ena ci-tva madh-tva drsh-ti
```
Interpretation:
1. Load `data` dataset
2. Bind `region` as grouping parameter
3. Group by region
4. Average values per region
5. Render results

### 3. Multiple Instruments
```
sales-at region-ena revenue-ena ci-tva yuj-tva drsh-ti
```
Interpretation:
1. Load `sales` dataset
2. Bind `region` and `revenue` as parameters
3. Group by region+revenue
4. Sum aggregates
5. Render results

### 4. Projection
```
sales-at columns-ena adhyaya-tva drsh-ti
```
Interpretation:
1. Load `sales` dataset
2. Bind `columns` as projection list
3. Select specified columns
4. Render results

---

## Architecture Overview

```
Input Program
  ↓ Lexer (src/lexer.rs)
Token Stream [Morpheme, Verb, Particle]
    ↓ Semantic Analyzer (src/semantic.rs)
Karaka Dependency Graph
    ↓ 3-Phase Parsing:
       Phase 1: Extract source (-at)
       Phase 2: Accumulate instruments (-ena)
       Phase 3: Process operations (-tva, -ti)
Semantic Graph (src/graph.rs)
    ↓ Anuvrtti Propagation
Context-Enriched Graph
    ↓ Topological Sort
Execution Plan
    ↓ (v0.3: Polars Lowering)
LazyFrame → Arrow → Results
```

---

## Graph Node Types

| Node Type | Purpose | Example |
|-----------|---------|---------|
| `Source` | Data entry | Load dataset |
| `Filter` | Row selection | chid operation |
| `GroupBy` | Aggregation key | ci operation |
| `Aggregate` | Value computation | yuj, madh, etc. |
| `Project` | Column selection | adhyaya operation |
| `Render` | Output/collection | drsh operation |

---

## Edge Types (Karaka Relations)

| Edge Type | Meaning | Used For |
|-----------|---------|----------|
| `Source` | Apādāna (origin) | Connect instruments to source |
| `Instrument` | Karaṇa (parameter) | Bind context parameters |
| `Object` | Karma (target) | Output bindings (v0.3) |
| `Scope` | Adhikarana (scope) | Scope inheritance (entry/exit) |

---

## File Structure

```
src/
  dhatu.rs         - Operation definitions (15+ dhatus)
  graph.rs         - Semantic graph IR + propagation
  lexer.rs         - ASCII tokenizer (no regex)
  semantic.rs      - Semantic analyzer (3-phase parser)
  main.rs          - CLI/REPL interface
  lib.rs           - Module exports

docs/
  SEMANTIC_GRAPH_ARCHITECTURE.md   - Full design
  MIGRATION_v0.1_TO_v0.2.md        - v0.1 → v0.2 guide
  RELEASE_NOTES_v0.2.md            - Features & changes
  INDEX_v0.2.md                    - Complete index

tests/  (25+ tests, all passing)
```

---

## Key Concepts

### Morphology-Driven Semantics
- Suffixes encode semantic roles, not positional arguments
- `-at` → source, `-ena` → parameter, `-tva` → lazy, `-ti` → terminal
- No parentheses, no commas, no punctuation

### Semantic Graph
- Explicit representation of dependencies
- Karaka labels on edges = semantic relationships
- Enables optimizer visibility (v0.3)

### Anuvrtti (Inheritance)
- Context flows automatically through DAG
- No parameter re-declaration
- Reduces redundancy, prevents bugs

### Asiddha (Deferred Execution)
- `-tva` = lazy evaluation
- `-ti` = collect and execute
- Pipelines don't execute until terminal

---

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| No source | `-at` missing | Add source dataset name |
| Invalid token | Misspelled morpheme | Check spelling |
| Unknown operation | Typo in dhatu | Use supported operations |
| Missing terminal | No `-ti` | Add `-ti` at end |
| Context error | Instrument not bound | Add `-ena` before operation |

---

## Performance Tips

- Use `-tva` for lazy evaluation
- Aggregate before rendering
- Filter early (v0.3 predicate pushdown)
- Group by low-cardinality columns first

---

## Version History

| Version | Date | Highlights |
|---------|------|-----------|
| 0.1.0 | May 15 | Initial AST compiler |
| 0.1.1 | May 20 | Added select (adhyaya) |
| 0.2.0 | May 29 | Semantic graph refactor |
| 0.3.0 | Q3 2026 | Optimizer + Polars lowering |
| 0.4.0 | Q4 2026 | Distributed execution |
| 0.5.0 | Q1 2027 | LLVM backend |
| 1.0.0 | Q2 2027 | Stable production |

---

## Support

- **Documentation**: See docs/ directory
- **Examples**: Run `examples` in REPL
- **Help**: Run `help` in REPL
- **Tests**: `cargo test --release`
- **Build**: `cargo build --release`

---

## Key Files to Study

1. **src/graph.rs** - How semantic graph works
2. **src/semantic.rs** - How Anuvrtti propagates
3. **SEMANTIC_GRAPH_ARCHITECTURE.md** - Theory + examples
4. **src/main.rs** - How to use the CLI

---

**PANINI-RS v0.2 - Production Ready**

For complete details, see SEMANTIC_GRAPH_ARCHITECTURE.md
