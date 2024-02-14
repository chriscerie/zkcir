# zkcir

Zero knowledge proof circuits intermediate representation.

[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/chriscerie/zkcir/actions/workflows/test.yml/badge.svg
[actions-url]: https://github.com/chriscerie/zkcir/actions?query=workflow%3ACI+branch%3Amain

# Motivation

Zero knowledge proofs are implemented in various frameworks, each with its own unique syntax and features. This leads to expensive context switching for cryptographic security analysts. This IR enables security analysis over a framework-agnostic environment.

# CLI Usage

- Use rust nightly
  - `rustup install nightly`
  - `rustup default nightly`
- Install CLI
  - Run `cargo install --branch main --git https://github.com/chriscerie/zkcir zkcir-cli`
  - Alternatively clone repo and run `cargo install --path zkcir-cli`
- Clone and run on a plonky2 circuit like https://github.com/chriscerie/plonky2-example
  - Run `zkcir --json --source -- --example square_root`
- To see possible args, run `zkcir --help`

# Gotchas

- Detecting a value was randomly generated is naive and can false positive. It stores the randomly generated values and marks any value that matches as random.
- Frameworks are moving targets. See the forks for the actual versions of the target dependencies.
  - [plonky2](https://github.com/chriscerie/plonky2)

# Online Compiler - AWS Deployment

We offer a self-hosted web app solution. To use, first deploy the app with AWS CDK.

* Update `cdk` resources as needed to fit your scalability requirements
* Add `0 issue "amazon.com"` CAA to domain record
* Deploy CDK
  * While deploying, approve certificate request
* Get application load balancer's DNS name from console and add as CNAME record

Internally fargate will compile the rust project with patched dependencies and store the executable in S3. It then sends the presigned url to lambda without access to other services and outbound internet to act as a sandboxed environment. Lambda then gets the executable and runs it, returning the resulting IR.
