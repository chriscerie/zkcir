# zkcir

Zero knowledge circuits IR.

[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/chriscerie/zkcir/actions/workflows/test.yml/badge.svg
[actions-url]: https://github.com/chriscerie/zkcir/actions?query=workflow%3ACI+branch%3Amain

# CLI Usage

* Use rust nightly
    * `rustup install nightly`
    * `rustup default nightly`
* Install CLI
    * Run `cargo install --branch main --git https://github.com/chriscerie/zkcir zkcir-cli`
    * Alternatively clone repo and run `cargo install --path zkcir-cli`
* Clone https://github.com/chriscerie/plonky2-example
* Run `zkcir --example square_root`
