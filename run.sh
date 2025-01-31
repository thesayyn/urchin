set -o errexit -o nounset -o pipefail
(cd spine && cargo build)
cargo run --bin bazel -- shutdown
cargo run --bin bazel -- build :test --isatty
cargo run --bin bazel -- version --isatty