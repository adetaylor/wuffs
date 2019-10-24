// Copyright 2019 The Wuffs Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ----------------

// This program exercises the Rust Lzw decoder used by image-gif.
//
// Wuffs' C code doesn't depend on Rust per se, but this program gives some
// performance data for specific Rust LZW implementations. The equivalent
// Wuffs benchmarks (on the same test files) are run via:
//
// wuffs bench std/lzw
//
// To run this program, do "cargo run --release" from the parent directory (the
// directory containing the Cargo.toml file).

extern crate lzw;
extern crate rustc_version_runtime;

use std::time::Instant;

const ITERSCALE: u64 = 20;
const REPS: u64 = 5;

fn main() {
    let version = rustc_version_runtime::version();
    print!(
        "# Rust {}.{}.{}\n",
        version.major,
        version.minor,
        version.patch
    );
    print!("#\n");
    print!("# The output format, including the \"Benchmark\" prefixes, is compatible with the\n");
    print!("# https://godoc.org/golang.org/x/perf/cmd/benchstat tool. To install it, first\n");
    print!("# install Go, then run \"go get golang.org/x/perf/cmd/benchstat\".\n");

    let mut dst = vec![0u8; 64 * 1024 * 1024];

    // The various magic constants below are copied from test/c/std/lzw.c
    for i in 0..(1 + REPS) {

        bench(
            "decode_20k",
            include_bytes!("../../../test/data/bricks-gray.indexes.giflzw"),
            &mut dst,
            50, // iters_unscaled
            i == 0, // warm_up
        );
        // The following bench has input data which appears to confuse
        // the LZW decoder.
        //bench(
            //"decode_100k",
            //include_bytes!("../../../test/data/pi.txt.giflzw"),
            //&mut dst,
            //10, // iters_unscaled
            //i == 0, // warm_up
        //);
    }
}

fn bench(
    name: &str, // Benchmark name.
    src: &[u8], // Source data.
    dst: &mut[u8], // Destination scratch buffer
    iters_unscaled: u64, // Base number of iterations
    warm_up: bool,
) {
    let iters = iters_unscaled * ITERSCALE;
    let mut total_num_bytes = 0usize;

    let start = Instant::now();
    for _ in 0..iters {
        let n = decode(dst, src);
        total_num_bytes += n;
    }
    let elapsed = start.elapsed();

    let elapsed_nanos = (elapsed.as_secs() * 1_000_000_000) + (elapsed.subsec_nanos() as u64);
    let kb_per_s: u64 = (total_num_bytes as u64) * 1_000_000 / elapsed_nanos;

    if warm_up {
        return;
    }

    print!(
        "Benchmarkrust_lzw_decode_{:16}   {:8}   {:12} ns/op   {:3}.{:03} MB/s\n",
        name,
        iters,
        elapsed_nanos / iters,
        kb_per_s / 1_000,
        kb_per_s % 1_000
    );
}

// decode returns the number of bytes processed.
fn decode(dst: &mut [u8], src: &[u8]) -> usize {
    let codesize = src[0];
    let mut input = &src[1..];
    let mut out_bytes = 0usize;
    let mut decoder = lzw::Decoder::new(lzw::LsbReader::new(), codesize);
    loop {
        let (n, slice) = decoder.decode_bytes(input).unwrap();
        if n == 0 {
            break;
        }
        let this_slice_len = slice.len();
        dst[out_bytes..out_bytes+this_slice_len].copy_from_slice(slice);
        out_bytes += this_slice_len;
        input = &input[n..];
    }
    out_bytes
}
