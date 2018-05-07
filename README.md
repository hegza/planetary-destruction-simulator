## Install
Install Rust toolchain by following the instructions at [www.rustup.rs/]().

## Run
The project requires you to have the isosurface-library in the same parent
directory as this project. I'm using a custom version of isosurface locally.
This custom version is available as a fork at:
- [](https://github.com/hegza/isosurface)

Then:
```
cargo run
```

## Logging
`export RUST_LOG=warn,ds_sim=TRACE`

