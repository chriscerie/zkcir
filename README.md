# zkcir

Zero knowledge proof circuits IR.

[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/chriscerie/zkcir/actions/workflows/test.yml/badge.svg
[actions-url]: https://github.com/chriscerie/zkcir/actions?query=workflow%3ACI+branch%3Amain

# CLI Usage

- Use rust nightly
  - `rustup install nightly`
  - `rustup default nightly`
- Install CLI
  - Run `cargo install --branch main --git https://github.com/chriscerie/zkcir zkcir-cli`
  - Alternatively clone repo and run `cargo install --path zkcir-cli`
- Clone https://github.com/chriscerie/plonky2-example
- Run `zkcir -- --example square_root`

# Gotchas

- Detecting a value was randomly generated is naive and can false positive. It stores the randomly generated values and marks any value that matches as random.
- Frameworks are moving targets. See the forks for the actual versions of the target dependencies.
  - [plonky2](https://github.com/chriscerie/zkcir)

# AWS Deployment
* Add `0 issue "amazon.com"` CAA to domain record
* Deploy CDK
  * While deploying, approve certificate request
* Get application load balancer's DNS name from console and add as CNAME record
