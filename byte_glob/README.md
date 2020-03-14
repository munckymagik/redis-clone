# byte_glob

A port of the stringmatchlen glob style string matching algorithm from Redis.

Works with byte slices rather than strict utf-8 strings, so it can be used
to match binary data.

## Testing

```shell
cargo test
```

## Fuzz testing

Fuzz testing has the following dependencies:

* A nightly compiler
  ```shell
  rustup toolchain install nightly
  ```
* cargo-fuzz
  ```shell
  cargo install cargo-fuzz
  ```

```bash
# Change into the `fuzz` sub-directory
cd fuzz

# Start fuzzing
cargo +nightly fuzz run fuzz_target

# Run for 5 minutes
cargo +nightly fuzz run fuzz_target -- -max_total_time=300

# Set a timeout to catch inputs that cause a long running time.
# This will timeout if any example input takes longer than 10s to process
cargo +nightly fuzz run fuzz_target -- -timeout=10 -max_total_time=300
```

References:

* [Rust Fuzz Book](https://rust-fuzz.github.io/book/) - a guide on using `cargo fuzz`
* [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html) - a guide on fuzzing with `libFuzzer`, includes descriptions of the options and the output
