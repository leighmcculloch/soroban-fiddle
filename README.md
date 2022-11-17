# Soroban Fiddle

https://leighmcculloch.github.io/soroban-fiddle

Web frontend-only application that displays data on the [Soroban] [Futurenet] network.

<img width="1465" alt="Screenshot 2022-11-17 at 12 06 05 AM" src="https://user-images.githubusercontent.com/351529/202390826-da662055-300b-4237-8454-dd4a496237d8.png">

## Features

- Displays deploys/invokes.
- For invokes it will show you: args, results, footprints, events.
- For deploys it will show you:
    - Rust interface for the contract. (types + functions)
    - Let you download the .wasm file.
- You can also simulate invoking functions (the ui is pretty horrible for now).
- Invoking a function uses the current state of the network, so if you go to the
URL below, click View next to deploy of ba989ae, and invoke the increment
function it should show you a result like 4 because people have already
incremented it a few times.
- Invocations are simulated in the browser and not sent to network.

All this is possible because:
- The Rust crates that power [Soroban] and related tooling works in WASM in the
browser. This application specifically uses `stellar-xdr`, `soroban-spec` and
`soroban-env-host`.
- [Horizon] exposes a stream of deployments and invocations via a HTTP API.

## Forked Dependencies

Uses a fork of the `backoff` crate, at
https://github.com/leighmcculloch/ihrwein--backoff/tree/glootimers.

[Soroban]: https://soroban.stellar.org
[Futurenet]: https://soroban.stellar.org/docs/networks/futurenet
[Horizon]: https://horizon-futurenet.stellar.org
