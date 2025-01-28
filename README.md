# Urchin

A Bazel launcher written in Rust. 

This is a combination of Bazelisk and Bazel's [own](https://github.com/bazelbuild/bazel/tree/3f9d80c35d88a280c2d53682f3d201b4733a3fff/src/main/cpp) launcher. 


# Run it yourself

Here's the command: `cargo run --bin bazel -- build :test --isatty`

```
thesayyn@Sahins-MacBook-Pro-2 urchin % cargo run -- build :test --isatty 
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/urchin build ':test' --isatty`
INFO: Analyzed target //:test (4 packages loaded, 7 targets configured).
INFO: Found 1 target...
Target //:test up-to-date (nothing to build)
INFO: Elapsed time: 0.713s, Critical Path: 0.03s
INFO: 1 process: 1 internal.
INFO: Build completed successfully, 1 total action
```