# `::repr_c`

# ⚠️ WIP ⚠️

This is currently still being developed and at an experimental stage, hence its
not being published to crates.io yet.

## Prerequisites

Minimum Supported Rust Version: `1.43.0`

## Quickstart

### `Cargo.toml`

Edit your `Cargo.toml` like so:

```toml
[package]
name = "crate_name"
version = "0.1.0"
edition = "2018"

[lib]
crate-type = ["staticlib"]

[dependencies]
repr_c = { git = "https://github.com/getditto/rust-repr_c.git", features = ["proc_macros"] }

[features]
c-headers = ["repr_c/headers"]
```

### `src/lib.rs`

```rust
use ::repr_c::prelude::*;

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub
struct Point {
    x: f64,
    y: f64,
}

#[ffi_export]
fn mid_point (
    left: &'_ Point,
    right: &'_ Point,
) -> Point
{
    Point {
        x: (left.x + right.x) / 2.,
        y: (left.y + right.y) / 2.,
    }
}

#[ffi_export]
fn print_point (point: &'_ Point)
{
    println!("{:?}", point);
}

#[::repr_c::cfg_headers]
#[test]
fn generate_headers () -> ::std::io::Result<()>
{
    ::repr_c::headers::builder()
        .to_file("rust_points.h")?
        .generate()
}
```

### Compilation & header generation

```shell
# Compile the C library (in `target/{debug,release}/libcrate_name.ext`)
cargo build # --release

# Generate the C header
cargo test --features c-headers -- generate_headers
```

<details><summary>Generated C header</summary>

```C
/*! \file */
/****************************************
 *                                      *
 *  File auto-generated by `::repr_c`.  *
 *                                      *
 *  Do not manually edit this file.     *
 *                                      *
 ****************************************/

#ifndef __RUST_CRATE_NAME__
#define __RUST_CRATE_NAME__

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    double x;

    double y;
} Point_t;

Point_t mid_point (
    Point_t const * left,
    Point_t const * right);

void print_point (
    Point_t const * point);


#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* __RUST_CRATE_NAME__ */
```

</details>

### Testing it

#### `main.c`

```C
#include <stdlib.h>

#include "rust_points.h"

int main (int argc, char const * const argv[])
{
    Point_t a = { .x = 84, .y = 45 };
    Point_t b = { .x = 0, .y = 39 };
    Point_t m = mid_point(&a, &b);
    print_point(&m);
    return EXIT_SUCCESS;
}
```

#### Compilation command

```bash
cc main.c -o main -L target/debug -l crate_name

# Now feel free to run the compiled binary
./main
```

which outputs:

```text
Point { x: 42.0, y: 42.0 }
```
