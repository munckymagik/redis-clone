#![no_main]
use libfuzzer_sys::{arbitrary, fuzz_target};
use byte_glob::glob;

#[derive(arbitrary::Arbitrary, Debug)]
struct GlobArgs {
    pub pattern: Vec<u8>,
    pub string: Vec<u8>,
}

fuzz_target!(|args: GlobArgs| {
    glob(&args.pattern, &args.string);
});
