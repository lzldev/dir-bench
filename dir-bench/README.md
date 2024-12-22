# dir-bench

[![Crates.io](https://img.shields.io/crates/v/dir-bench.svg)](https://crates.io/crates/dir-bench)
[![License](https://img.shields.io/crates/l/dir-bench)](https://opensource.org/licenses/MIT)

`dir-bench` provides a macro to generate benchmarks from files in a directory.

**NOTE**: currently rust only supports running benchmarks in the nigthly channel.

crate based on [dir-test](https://crates.io/crates/dir-test)

## Usage

Add the following dependency to your `Cargo.toml`.

```toml
[dev-dependencies]
dir-bench = "0.1.0"
```

### Basic Usage

```rust, no_run
extern crate test;

use test::Bencher;
use dir_bench::{dir_bench, Fixture};

#[dir_bench(
    dir: "$CARGO_MANIFEST_DIR/fixtures",
    glob: "**/*.txt",
)]
fn benchmark(b: &mut Bencher,fixture: Fixture<&str>) {
    // The file content and the absolute path of the file are available as follows.
    let content = fixture.content();
    let path = fixture.path();

    // Setup your benchmark
    // ...

    b.iter(|| {
        // Write your benchmark code here
        // ...
    })
}
```

Assuming your crate is as follows, then the above code generates two benchmarks
cases `mybenchmark__foo()` and `mybenchmark__fixtures_a_bar()`.

```text
my-crate/
├─ fixtures/
│  ├─ foo.txt
│  ├─ fixtures_a/
│  │  ├─ bar.txt
├─ src/
│  ├─ ...
│  ├─ lib.rs
├─ Cargo.toml
├─ README.md
```

🔽

```rust, no_run
#[bench]
fn mybenchmark__foo(b: &mut Bencher) {
    //...
    mybenchmark(b,fixture);
}

#[bench]
fn mybenchmark__fixtures_a_bar(b: &mut Bencher) {
    //...
    mybenchmark(b,fixture);
}
```

**NOTE**: The `dir` argument must be specified in an absolute path because
of the limitation of the current procedural macro system. Consider using
environment variables, `dir-bench` crate resolves environment variables
internally.

### Benchmark Attributes

Benchmark attributes can specified by the `dir_bench_attr` attribute. The
attributes inside `dir_bench_attr` are applied to the all generated benchmarks.

```rust, no_run
use dir_bench::{dir_bench, Fixture};

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/fixtures",
    glob: "**/*.txt",
)]
#[dir_bench_atrr(
    #[wasm_bindgen_test]
    #[cfg(target_family = "wasm")]
)]
fn wasm_test(fixture: Fixture<std::io::Result<String>>) {
    // ...
}
```
