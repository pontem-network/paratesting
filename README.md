# Paratesting

The cli-tool for testing parachains.

With maximal possible automatisation and simple configuration, but flexibility as priority in mind.

So with this tool you'll can run tests on you single node or parachain, send calls, check/watch state, read & check storage (raw or specific runtime-module), etc.

<!-- TODO: describe examples of test-cases for parachains or individual nodes (standalone chaines) -->


__Project is in heavy development state.__
This is a first draft and partially implemented PoC.

Any contributions are highly welcome and will be appreciated ❤️.


## Usage

Prerequisites:
- to run nodes one of the following options:
	- installed [polkadot-launch][] bin package (optional but gives easy setup)
		- polkadot-launch config for parachain in `../../substrate/node`
	- other tool or script

For example run it with Pontem local parachain:
1. download the release-build for you OS [here][pontem-release] or [build it manually][pontem-readme].
1. use [instructions][pontem-readme] about prerequisites such as [polkadot-launch][] optionally.
1. [download][polkadot-releases] or build manually polkadot-node and put it to `nodes/`-directory as specified in example test-suit in `examples/cases/case-test.yaml`
1. edit paths & ports in `case-test.yaml` and `pontem/launch-config.json` if needed
1. run `paratesting -i examples/cases`


### UI

Just one parameter is required - `--input` path to directory with tests or the one test file.

Test is a yaml formatted file looks like [this one](examples/cases/case-test.yaml).

<!-- TODO: describe the format -->


### Logging

Logging configurable as any other standard rust-log configuration with `env`-variables: `RUST_LOG` & `RUST_LOG_STYLE`.

We're recommend `RUST_LOG=trace,async_io=info,polling=info` for this project now.

See [documentation](https://docs.rs/env_logger/0.9.0/env_logger/#enabling-logging) for more info.


#### Github CI environment

There is special logging & reporting configuration implemented as feature `github`. If enabled, logs throws in [gh-format][].


### Evaluation

As planned almost all fealds about keys, values, arguments or success criteria (conditions) will be evaluated. So you'll be able use some simple expressions like `success: account("//Alice").balance.free > 42`.



## Development

Prerequisites:
- Rust toolchain (nightly channel)


### How to build

Just run `cargo build`.

Supported runtimes:

<!-- TODO: third column with specific versions -->

| Runtime     | Feature            |
|-------------|--------------------|
| Pontem      | `runtime-pontem`   |
| Rococo      | `runtime-rococo`   |
| Polkadot    | `runtime-polkadot` |



#### How to build especially with/for metadata by your node, where publicly used custom types

Prerequisites:
- [subxt][] executable to generate metadata for custom runtime types support

1. Query node metadata and generate types in rust:
	1. start your node
	1. query node runtime metadata:
		```
		#                             node api url & port     output
		subxt metadata -f=bytes --url=http://127.0.0.1:9933 > metadata/custom.scale
		```
		See [docs](https://github.com/paritytech/subxt/blob/master/cli/README.md) for that tool.
	1. generate sources: `subxt codegen -f ./metadata.scale | rustfmt --emit=stdout > metadata.rs`
	1. replace client/gen/pontem.rs with generated code and build with feature `runtime-pontem` or
	1. ~put generated to client/gen/custom.rs (rel to root of paratesting)~ _currently planned but not implemented yet_.
	1. ~pub metadata.scale to metadata/custom.scale and build with feature `runtime-custom`~
1. Build with ^mentioned feature

Dynamic (any) runtime types and metadata support without rebuild requirement are planned and probably will.



#### Tests

There are two kinds of tests:
- standard tests with `cargo test --all`
- system <!-- is it a "system" tests? --> tests, run `cargo run -- -i tests/assets`




[polkadot-launch]: https://github.com/paritytech/polkadot-launch
[subxt]: https://github.com/paritytech/subxt
[gh-format]: https://docs.github.com/en/actions/learn-github-actions/workflow-commands-for-github-actions#setting-a-warning-message
[pontem-release]: https://github.com/pontem-network/pontem/releases
[pontem-readme]: https://github.com/pontem-network/pontem/blob/master/README.md#build
[polkadot-launch]: https://github.com/paritytech/polkadot-launch
[polkadot-releases]: https://github.com/paritytech/polkadot/releases
