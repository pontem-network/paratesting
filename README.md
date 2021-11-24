# Paratesting

Framework for testing parachains.

Project is in highly development state.

## Usage

Prerequisites:
- installed [polkadot-launch][] bin package (optional but gives easy setup)
	- polkadot-launch config for parachain in `../../substrate/node`
- [subxt][] executable to generate metadata for custom runtime types support

[polkadot-launch]: https://github.com/paritytech/polkadot-launch
[subxt]: https://github.com/paritytech/subxt

Run `node ./main.js ./examples/cases` to test example cases.








#### Generate metadata

```
#                             node api url & port     output
subxt metadata -f bytes --url http://127.0.0.1:9933 > metadata/name.scale
```
