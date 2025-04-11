[![Docs](https://img.shields.io/badge/docs-docs.rs-ff69b4.svg)](https://docs.rs/crossplane-types/)
[![crates.io](https://img.shields.io/crates/v/shuttle-next.svg)](https://crates.io/crates/crossplane-types)
[![License](https://img.shields.io/badge/license-apache-blue.svg)](https://raw.githubusercontent.com/shuttle-hq/crossplane-types-rs/main/LICENSE)

> **Warning**: EXPERIMENTAL. **Not endorsed for production use**.

> **Note**: While the aspiration is to eventually become the "official" Rust
> bindings for [Crossplane](https://crossplane.io/) resources, [UpBound](https://www.upbound.io/)
> (the maintainer of Crossplane) has not yet (and may never) officially endorsed
> it so this crate should be considered "unofficial" until further notice.

# Crossplane Resource Types (Rust)

This project provides [Rust](https://rust-lang.org) bindings for Crossplane-managed [Kubernetes](https://kubernetes.io/) resources.

> [!NOTE]
> Currently supports resources from:
> - [AWS Provider Family v1.21.1](https://github.com/crossplane-contrib/provider-upjet-aws/releases/tag/v1.21.1)

## Usage

Basic usage involves using a [kube-rs](https://github.com/kube-rs/kube)
[Client](https://docs.rs/kube/latest/kube/struct.Client.html) to perform create, read, update
and delete (CRUD) operations on [Crossplane resources](https://marketplace.upbound.io/providers).
You can either use a basic `Client` to perform CRUD operations, or you can build a
[Controller](https://kube.rs/controllers/intro/). See the `crossplane-types/examples/` directory
for detailed (and specific)usage examples.

## Development

This project uses [Kopium](https://github.com/kube-rs/kopium) to automatically generate API bindings from upstream
Crossplane provider repositories. Make sure you install `kopium` locally in order to run the generator:

```console
$ cargo install kopium --version 0.21.1
```

After which you can run the `update.sh` script:

```console
$ ./update.sh
```

Check for errors and/or a non-zero exit code, but upon success you should see
updates automatically generated for code in the `crossplane-types/src/resources` directory
which you can then commit.

## Contributions

Contributions are welcome, and appreciated! In general (for larger changes)
please create an issue describing the contribution needed prior to creating a
PR.

For development support we do have an org-wide [Discord server](https://discord.gg/shuttle),
but please note that for this project in particular we prefer questions be posted in
the [discussions board](https://github.com/shuttle-hq/crossplane-types-rs/discussions).
