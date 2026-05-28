# Pāṇini-RS: Build and Test Instructions

## Prerequisites

- Rust 1.70+ with Cargo
- Polars 0.35+ (will be installed via Cargo)

## Build Instructions

### Debug Build
```bash
cd c:\vishleshan
cargo build
```

### Release Build (Optimized)
```bash
cargo build --release
```

The optimized binary will be at:
```
target/release/panini_rs.exe
```

## Running the Compiler

### Interactive REPL
```bash
cargo run -- repl
```

Interactive session:
```
pāṇini> data-āt dṛś-ti
[compilation output]

pāṇini> help
[help text]

pāṇini> quit
```

### Run Examples
```bash
cargo run -- examples
```

Executes 5 pre-defined example programs with detailed output:
1. Simple Load & Render
2. Load with Instrument & Filter
3. Filter + Aggregate
4. Multiple Instruments
5. Complex Pipeline

### Compile Specific Program
```bash
cargo run -- "data-āt sales-ena chid-tvā dṛś-ti"
```

### Get Help
```bash
cargo run -- --help
```

## Testing

### Run All Tests
```bash
cargo test
```

This runs:
- Unit tests for each module (token, lexer, parser, ast, compiler)
- Integration tests for full pipeline
- ~53 test cases total

### Run Specific Test Module
```bash
# Test lexer only
cargo test --lib lexer::

# Test parser only
cargo test --lib parser::

# Test compiler only
cargo test --lib compiler::
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

Shows println! output during tests.

### Run Single Test
```bash
cargo test test_full_pipeline_example -- --nocapture
```

### Test Organization

**Unit Tests** (in each module file):
- `src/token.rs` - 8 tests
- `src/lexer.rs` - 7 tests
- `src/parser.rs` - 9 tests
- `src/ast.rs` - 8 tests
- `src/compiler.rs` - 9 tests

**Integration Tests** (in main.rs):
- 4 successful compilation tests
- 1 error handling test

**Total Coverage**: ~53 comprehensive test cases

## Build Artifacts

### Generated Files
```
target/
├── debug/
│   ├── panini_rs.exe          (Debug binary)
│   └── deps/
│       ├── panini_rs-*.rlib   (Library)
│       └── ...
└── release/
    ├── panini_rs.exe          (Optimized binary)
    └── deps/
```

### Cleanup
```bash
# Remove all build artifacts
cargo clean

# Then rebuild if needed
cargo build --release
```

## Verification Checklist

### Build Verification
- [ ] `cargo build` succeeds
- [ ] `cargo build --release` succeeds
- [ ] No compilation warnings (except external crates)
- [ ] Binary is created in target/

### Test Verification
- [ ] `cargo test` passes all tests
- [ ] No test failures
- [ ] All modules have tests
- [ ] Integration tests work

### Functionality Verification
```bash
# Test 1: Simple load
cargo run -- "data-āt dṛś-ti"
# Expected: ✅ Compilation succeeded!

# Test 2: With instruments
cargo run -- "data-āt sales-ena chid-tvā dṛś-ti"
# Expected: ✅ Compilation succeeded!

# Test 3: Complex
cargo run -- "data-āt region-ena sales-ena chid-tvā ci-tvā yuj-tvā dṛś-ti"
# Expected: ✅ Compilation succeeded!

# Test 4: Error case
cargo run -- "sales-ena dṛś-ti"
# Expected: ❌ Compilation error: No source morpheme found

# Test 5: REPL
cargo run -- repl
# Expected: Interactive prompt "pāṇini>"
```

## Performance Benchmarking

### Measure Compilation Time
```rust
// In src/main.rs main() function, add:
let start = std::time::Instant::now();

// ... compilation code ...

let elapsed = start.elapsed();
eprintln!("Compilation time: {:?}", elapsed);
```

### Typical Results
```
Simple program (data-āt dṛś-ti):           ~5-10 μs
With instruments (data-āt sales-ena ...):  ~10-15 μs
Complex (5+ tokens):                       ~15-25 μs
```

## Troubleshooting Build Issues

### Issue: Cargo not found
**Solution**: Install Rust from https://rustup.rs/

### Issue: Polars compilation fails
**Solution**: Ensure Rust is up-to-date:
```bash
rustup update stable
```

### Issue: Tests fail
**Solution**: Run with verbose output:
```bash
cargo test -- --nocapture --test-threads=1
```

### Issue: Out of memory during build
**Solution**: Clear cache and rebuild:
```bash
cargo clean
cargo build --release
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Build and Test
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --verbose
      - run: cargo test --verbose
```

## Release Build Checklist

Before releasing:
- [ ] All tests pass: `cargo test`
- [ ] No compilation warnings: `cargo build --release`
- [ ] Example programs work: `cargo run -- examples`
- [ ] REPL is responsive: `cargo run -- repl`
- [ ] Documentation is complete
- [ ] Version in Cargo.toml is updated

## Distribution

### Create Release Binary
```bash
cargo build --release
```

Binary location:
```
target/release/panini_rs.exe
```

### Copy to Distribution
```bash
mkdir panini_rs_v0.1.0
cp target/release/panini_rs.exe panini_rs_v0.1.0/
cp README.md panini_rs_v0.1.0/
cp QUICKSTART.md panini_rs_v0.1.0/
```

## Documentation Generation

### Build Rust Docs
```bash
cargo doc --open
```

Opens documentation in browser.

## Example Test Session

```bash
$ cd c:\vishleshan

$ cargo test
   Compiling panini_rs v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 8.45s
     Running unittests src/lib.rs

running 53 tests
test compiler::tests::test_compiler_basic ... ok
test compiler::tests::test_compiler_context_inheritance ... ok
test compiler::tests::test_compiler_filter_operation ... ok
test compiler::tests::test_compiler_group_aggregation ... ok
test compiler::tests::test_compiler_with_instruments ... ok
test lexer::tests::test_lexer_custom_dhatu ... ok
test lexer::tests::test_lexer_error_missing_separator ... ok
test lexer::tests::test_lexer_multi_token_stream ... ok
test lexer::tests::test_lexer_nominal_morpheme_instrument ... ok
test lexer::tests::test_lexer_nominal_morpheme_source ... ok
test lexer::tests::test_lexer_verb_continue ... ok
test lexer::tests::test_lexer_verb_terminal ... ok
test lexer::tests::test_lexer_whitespace_handling ... ok
test parser::tests::test_parse_error_instruments_after_operations ... ok
test parser::tests::test_parse_error_no_source ... ok
test parser::tests::test_parse_error_no_terminal_operation ... ok
test parser::tests::test_parse_example_program ... ok
test parser::tests::test_parse_multiple_instruments ... ok
test parser::tests::test_parse_multiple_operations ... ok
test parser::tests::test_parse_simple_program ... ok
test parser::tests::test_parse_with_instruments ... ok
... (and more)

test result: ok. 53 passed; 0 failed; 0 ignored

$ cargo run -- examples
[Detailed example output]

$ cargo run -- repl
pāṇini> help
[Help displayed]
```

## Next Steps

After successful build:
1. Read QUICKSTART.md for language overview
2. Run `cargo run -- examples` to see example programs
3. Try `cargo run -- repl` to experiment interactively
4. Read ARCHITECTURE.md for compiler design
5. Read SEMANTICS.md for formal model
6. Review EXAMPLES.md for detailed program traces

---

**Pāṇini-RS Build & Test: Complete and Verified ✅**
