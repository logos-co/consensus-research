# Consensus Research

## Project Structure

* `consensus`: Consensus implementation libraries
  * `snowball`: Snowball implementation
  * `claro`: Claro implementation
* `prototypes`: Simulations and experiments related libraries and binaries
  * `consensus-simulations`: Consensus simulations app

## Build & Test

Minimal Rust supported version: `1.63`

When in development, please, use `cargo clippy` to build the project. Any warning is promoted to an error in our CI.

* Use `cargo test` for executing tests, and `cargo test -- --nocapture` for seeing test outputs.
* Use `cargo run --exampel {example_name}` to run an example.

### Build Documentation

Simply run `cargo doc --open --no-deps` to build and access a copy of the generated documentation.
