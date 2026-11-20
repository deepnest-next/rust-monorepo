# Building rust-monorepo

Clone with submodules (`crates/libs/babushka` is
[rust-babushka-nfp](https://github.com/deepnest-next/rust-babushka-nfp)):

```bash
git clone --recurse-submodules https://github.com/deepnest-next/rust-monorepo
# or, in an existing clone:
git submodule update --init --recursive
```

## External dependencies

| Needed for | What | Debian/Ubuntu | Fedora | macOS | Windows |
|---|---|---|---|---|---|
| `babushka` (`build.rs` compiles `gpc.c`), Clipper2 (C++) | C/C++ compiler | `build-essential` | `gcc gcc-c++` | Xcode Command Line Tools | MSVC Build Tools |
| `babushka` (`build.rs` runs `bindgen`) | clang + libclang | `clang libclang-dev` | `clang clang-devel` | Xcode Command Line Tools | LLVM (`winget install LLVM.LLVM`), set `LIBCLANG_PATH` |
| Loading/testing the N-API addons (`crates/npm/*`, `geometry-utils`, `deepnest-lib`); not needed for `cargo build` | Node.js | `nodejs` | `nodejs` | `brew install node` | Node.js installer |

Without libclang the build stops in babushka's `build.rs` with
`Unable to generate bindings: ... 'stddef.h' file not found`.

## Build and test

```bash
cargo build --workspace
# geometry-utils and deepnest-lib link against Node's N-API (napi_* symbols); their tests
# only link inside Node, so `cargo test` excludes them:
cargo test --workspace --exclude geometry-utils --exclude deepnest-lib
```

No system packages? Build in a throwaway container:

```bash
docker run --rm -v "$PWD":/src:ro rust:1-bookworm bash -c \
  'apt-get update && apt-get install -y clang libclang-dev && cp -r /src /w && cd /w && cargo build --workspace'
```
