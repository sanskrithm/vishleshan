# Pāṇini-RS: Formal Semantics

## Overview

Pāṇini-RS implements **morphological semantics** where meaning is encoded in morpheme structure rather than positional syntax. This document describes the formal semantic model.

## 1. KĀRAKA SEMANTICS (Case-Driven Meaning)

### Apādāna (Source/Origin) - `-āt`

**Morphological Form**: `identifier-āt`

**Semantic Function**: Source binding

**Formal Definition**:
```
Apādāna(id) = bind_source(id)
```

**Semantics**:
- Designates the origin of data in the pipeline
- Must appear exactly once, as the first morpheme
- Establishes the initial dataset for the computation

**Examples**:
```
data-āt       ≡ LoadSource("data")
sales-āt      ≡ LoadSource("sales")
customers-āt  ≡ LoadSource("customers")
```

**Type Signature**:
```
Apādāna: String → QueryStep
Apādāna(source) = LoadSource(source)
```

### Karaṇa (Instrument/Means) - `-ena`

**Morphological Form**: `identifier-ena`

**Semantic Function**: Parameter binding with inheritance (Anuvṛtti)

**Formal Definition**:
```
Karaṇa(id) = add_to_context(id)

where context is inherited by all subsequent operations
```

**Semantics**:
- Declares a parameter available to all subsequent operations
- Implements **Anuvṛtti** (automatic state inheritance)
- Once declared, does not need explicit re-binding
- Multiple instruments accumulate in context

**Examples**:
```
sales-ena      ≡ context.add("sales")
region-ena     ≡ context.add("region")
discount-ena   ≡ context.add("discount")
```

**Type Signature**:
```
Karaṇa: String → (Context → Context)
Karaṇa(param)(ctx) = ctx ∪ {param}
```

**Anuvṛtti Formalization**:
```
LetContext(P, C) =
  case P of
    []                    → C
    (param-ena : rest)   → LetContext(rest, C ∪ {param})
    (op : rest)          → CompileOp(op, C) ++ LetContext(rest, C)
```

## 2. SŪTRA SEMANTICS (Control Flow)

### Temporal Continuation - `-tvā`

**Morphological Form**: `dhātu-tvā`

**Semantic Function**: Lazy operation (pipeline continuation)

**Formal Definition**:
```
-tvā = DeferExecution
     = AddToLazyPipeline
     = continue_next_operation
```

**Semantics**:
- Marks an operation as lazy (deferred)
- Adds operation to the computation graph
- Execution is deferred until a terminal operation
- Allows pipeline fusion and optimization

**Execution Model**:
```
lazy_state = lazy_state.apply(operation)  // Lazy (no execution)
```

**Examples**:
```
chid-tvā  ≡ Filter(lazy=true)
yuj-tvā   ≡ Aggregate(lazy=true)
ci-tvā    ≡ GroupBy(lazy=true)
```

### Terminal Execution - `-ti`

**Morphological Form**: `dhātu-ti`

**Semantic Function**: Terminal operation (trigger execution)

**Formal Definition**:
```
-ti = ExecuteNow
    = TriggerEvaluation
    = collect_and_output
```

**Semantics**:
- Marks operation as terminal
- Must be the final operation
- Triggers lazy pipeline evaluation
- Results in visible output

**Execution Model**:
```
result = lazy_state.execute()  // Eager evaluation
output(result)                  // Display result
```

**Examples**:
```
dṛś-ti   ≡ Render(execute=true)
ci-ti    ≡ GroupBy(execute=true)  [Non-standard but valid]
```

**Constraint**:
```
∀ program p: final_operation(p).execution = Terminal
```

## 3. DHĀTU SEMANTICS (Operation Meanings)

### chid (Cut/Filter)

**Linguistic Meaning**: To cut, divide, separate

**Programming Meaning**: Filter operation

**Formal Definition**:
```
chid(context) = filter(x | ∀c ∈ context: x.c ≠ NULL)
```

**Polars Lowering**:
```rust
|df| → df.filter(
    col(context[0]).gt(lit(0))
        .and(col(context[1]).gt(lit(0)))
        ...
)
```

**Semantics**:
- Applies predicate filtering
- Uses inherited instruments as filter conditions
- Requires non-empty context
- Null-checks on all context parameters

**Examples**:
```
data-āt sales-ena chid-tvā dṛś-ti
  ↓
  Filter: sales IS NOT NULL

data-āt region-ena sales-ena chid-tvā dṛś-ti
  ↓
  Filter: region IS NOT NULL AND sales IS NOT NULL
```

### ci (Gather/Group)

**Linguistic Meaning**: To gather, collect, heap

**Programming Meaning**: Group-by operation

**Formal Definition**:
```
ci(context) = group_by(columns = context)
```

**Polars Lowering**:
```rust
|df| → df.group_by(context).agg([
    col(context[0]).count().alias("count"),
    ...
])
```

**Semantics**:
- Groups data by instrument columns
- Uses all instruments in context
- Aggregates within groups
- Requires non-empty context

**Examples**:
```
data-āt region-ena ci-tvā dṛś-ti
  ↓
  GroupBy: region

data-āt region-ena sales-ena ci-tvā dṛś-ti
  ↓
  GroupBy: region, sales
```

### yuj (Join/Aggregate)

**Linguistic Meaning**: To join, unite, combine

**Programming Meaning**: Aggregation operation

**Formal Definition**:
```
yuj(context) = aggregate(sum(c) | c ∈ context)
```

**Polars Lowering**:
```rust
|df| → df.agg([
    col(context[0]).sum().alias(format!("sum_{}", context[0])),
    col(context[1]).sum().alias(format!("sum_{}", context[1])),
    ...
])
```

**Semantics**:
- Numerically aggregates columns
- Sums all numeric columns in context
- Produces scalar or grouped results
- Requires non-empty context

**Examples**:
```
data-āt sales-ena yuj-tvā dṛś-ti
  ↓
  Aggregate: SUM(sales)

data-āt region-ena sales-ena ci-tvā yuj-tvā dṛś-ti
  ↓
  GroupBy(region) → SUM(sales) per region
```

### dṛś (See/Render)

**Linguistic Meaning**: To see, perceive, view

**Programming Meaning**: Render/output operation

**Formal Definition**:
```
dṛś() = collect() >> print()
```

**Polars Lowering**:
```rust
|lazy_df| → {
    let df = lazy_df.collect();
    println!("{:?}", df);
}
```

**Semantics**:
- Terminal operation (always -ti)
- Evaluates the lazy pipeline
- Outputs result to stdout
- Cannot be lazy (-tvā)

**Constraint**:
```
∀ program p: dṛś ∈ p ⟹ dṛś.execution = Terminal
```

**Examples**:
```
dṛś-ti    ✓ (standard: render and output)
dṛś-tvā   ✗ (invalid: dṛś must be terminal)
```

## 4. ANUVṚTTI FORMALIZATION (State Inheritance)

### Definition

**Anuvṛtti** (Sanskrit: अनुवृत्ति, "repetition" or "carrying over") is the principle that once a grammatical element is introduced, it carries through all subsequent elements until explicitly changed.

### Formal Model

```
CONTEXT := P(String)  -- Powerset of string identifiers

Program := (source: String, instruments: List<String>, ops: List<Operation>)

Semantics[[·]] : Program → QueryPlan

Semantics[[P]] = 
  let ctx₀ = ∅
      ctx₁ = foldl (λctx id. ctx ∪ {id}) ctx₀ P.instruments
      steps = [LoadSource(P.source)] ++ compileOps(P.ops, ctx₁)
  in QueryPlan(steps, ctx₁)

compileOps : List<Operation> × Context → List<QueryStep>

compileOps([], ctx) = [Render]
compileOps([op], ctx) = [compileOp(op, ctx), Render]
compileOps([op : ops'], ctx) = [compileOp(op, ctx)] ++ compileOps(ops', ctx)

compileOp : Operation × Context → QueryStep

compileOp(Chid, ctx) = Filter(conditions(ctx))
compileOp(Ci, ctx) = GroupBy(ctx)
compileOp(Yuj, ctx) = Aggregate(sum, ctx)
compileOp(Drsh, ctx) = Render
```

### Inheritance Property

**Property 1 (Context Monotonicity)**:
```
∀ i ≤ j: context_at(i) ⊆ context_at(j)
```
Context only grows (instruments are added, never removed).

**Property 2 (Context Persistence)**:
```
param ∈ context_at(i) ⟹ ∀ j > i: param ∈ context_at(j)
```
Once added, parameter is available to all subsequent operations.

**Property 3 (Context Accessibility)**:
```
op_j can reference param ⟺ param ∈ context_before(op_j)
```
Operations can only use parameters in their context.

### Example Derivation

**Input**: `data-āt region-ena sales-ena chid-tvā yuj-tvā dṛś-ti`

**Derivation**:

```
ctx₀ = ∅

instruments = [region, sales]
ctx₁ = foldl add ctx₀ instruments
     = ∅ ∪ {region} ∪ {sales}
     = {region, sales}
     [Anuvṛtti: both now in context]

operations = [chid-tvā, yuj-tvā, dṛś-ti]

compileOp(chid, {region, sales})
  → Filter("region IS NOT NULL", "sales IS NOT NULL")
  [Uses inherited context]

compileOp(yuj, {region, sales})
  → Aggregate(sum, ["region", "sales"])
  [Uses inherited context]

compileOp(dṛś, {region, sales})
  → Render
  [Terminal: executes pipeline]

QueryPlan:
  context: {region, sales}
  steps: [
    LoadSource("data"),
    Filter(...),      [ctx = {region, sales}]
    Aggregate(...),   [ctx = {region, sales}]
    Render,
  ]
```

## 5. LAZY EVALUATION MODEL

### Lazy vs Eager Semantics

```
Lazy (-tvā):
  operation ∈ LazyPipeline
  no side effects
  can be fused/optimized

Eager (-ti):
  operation triggers evaluation
  side effects: print output
  pipeline materialized
```

### Pipeline Fusion

```
data-āt col-ena
  chid-tvā        -- Lazy: add to pipeline
  yuj-tvā         -- Lazy: add to pipeline  
  dṛś-ti          -- Eager: fuse all and execute

Fused Pipeline:
  df.filter(col("col") > 0)
    .agg(col("col").sum())
    .collect()
    .print()
```

### Execution Order

```
Operation Sequence:
  [chid-tvā, yuj-tvā, dṛś-ti]

Lazy Pipeline (deferred):
  df = df.filter(...)    ← not executed
  df = df.agg(...)       ← not executed

Terminal Operation (executed):
  result = df.collect()  ← NOW executes whole pipeline
  print(result)          ← side effect
```

## 6. TYPE SEMANTICS

### Nominal Types

```
Type := DataFrameType | ScalarType | VoidType

DataFrameType(cols: List<String>)
ScalarType(numeric | string)
VoidType (for Render operation)
```

### Operation Type Signatures

```
LoadSource : String → LazyFrame
Filter : LazyFrame × Context → LazyFrame
GroupBy : LazyFrame × Context → LazyFrame
Aggregate : LazyFrame × Context → LazyFrame
Render : LazyFrame → Void
```

## 7. ERROR SEMANTICS

### Parse-Time Errors

```
Error := SourceNotFirst
       | InstrumentAfterOp
       | NoTerminalOp
       | UnknownDhatu
       | InvalidUtf8

Semantics: Halt compilation, report diagnostic
```

### Compile-Time Errors

```
Error := ContextRequired
       | UnboundParameter
       | InvalidOperation

Semantics: Halt compilation, suggest fix
```

### Runtime Errors

```
Error := DatasetNotFound
       | ColumnNotFound
       | AggregationFailed

Semantics: Halt execution, report error location
```

## 8. EXAMPLE DERIVATIONS

### Example 1: Simple Render
```
Input: data-āt dṛś-ti

Parse:
  source = "data"
  instruments = []
  operations = [Operation(Drsh, Terminal)]

Compile:
  context = {}
  steps = [
    LoadSource("data"),
    Render
  ]

Semantics:
  df = load("data")
  print(df)
```

### Example 2: Filter Pipeline
```
Input: data-āt sales-ena chid-tvā dṛś-ti

Parse:
  source = "data"
  instruments = ["sales"]
  operations = [Operation(Chid, Continue), Operation(Drsh, Terminal)]

Compile:
  context after instruments: {sales}
  
  compileOp(chid, {sales}):
    Filter(col("sales") > 0)
  
  compileOp(drsh, {sales}):
    Render

  steps = [
    LoadSource("data"),
    Filter(col("sales") > 0),
    Render
  ]

Semantics:
  df = load("data").lazy()
  df = df.filter(col("sales") > 0)  ← lazy
  result = df.collect()              ← eager
  print(result)
```

### Example 3: Group & Aggregate
```
Input: data-āt region-ena sales-ena ci-tvā yuj-tvā dṛś-ti

Parse:
  source = "data"
  instruments = ["region", "sales"]
  operations = [
    Operation(Ci, Continue),
    Operation(Yuj, Continue),
    Operation(Drsh, Terminal)
  ]

Compile:
  context: {region, sales}
  
  steps = [
    LoadSource("data"),
    GroupBy([region, sales]),        ← uses context
    Aggregate(sum, [region, sales]), ← uses context
    Render
  ]

Semantics:
  df = load("data").lazy()
  df = df.group_by([col("region"), col("sales")])
           .agg([col("sales").sum()])  ← lazy
  result = df.collect()                 ← eager
  print(result)
```

## 9. SOUNDNESS & COMPLETENESS

### Soundness
```
Property: If program p compiles without error,
          then executing the compiled query plan succeeds
          (modulo data availability).
```

### Completeness
```
Property: For all well-formed Sanskrit morpheme sequences
          satisfying Pāṇini's rules, the compiler produces
          a valid query plan.
```

---

**Pāṇini-RS: Where Sanskrit Morphology Meets Formal Semantics**
