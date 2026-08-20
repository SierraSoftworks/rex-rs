# Rex
**Keep track of your ideas for things to do, and get handed a random one on demand**

[![License](https://img.shields.io/github/license/SierraSoftworks/rex-rs.svg?style=for-the-badge)](./LICENSE)
[![Coverage](https://img.shields.io/codecov/c/github/SierraSoftworks/rex-rs?style=for-the-badge)](https://app.codecov.io/gh/SierraSoftworks/rex-rs)

Rex keeps a list of the things you have been meaning to do, and when you cannot
decide, picks one for you. Ideas live in collections, which you can share with
other people.

Rex ships as **one binary**. The web interface is compiled to WebAssembly and
embedded in the executable, the database is SQLite compiled in alongside it, and
authentication is any standard OpenID Connect provider. There is nothing else to
install and nothing else to run.

## Running it

```bash
cargo run -p rex-server
```

With no `config.toml` present, Rex starts on `:8000` with an in-memory store and
no authentication, which is enough to look around. For anything you want to
keep, copy [`config.example.toml`](./config.example.toml) to `config.toml` and
edit it — every option is documented there. `REX_CONFIG` overrides the path.

> Running without `[web.auth.oidc]` means **every request is treated as the same
> local developer**. That is a development convenience, not a deployment option.

### With Docker

```bash
docker run -p 8000:8000 \
  -v "$PWD/config.toml:/config/config.toml:ro" \
  -v rex-data:/data \
  ghcr.io/sierrasoftworks/rex-rs:latest
```

## Building it

The server embeds `ui/dist` **at compile time**, so the interface has to be
built before the server is:

```bash
cd ui && trunk build && cd ..
cargo build --release -p rex-server
```

Building the server without doing that first produces a binary that serves a
plain error page where the interface should be. The build script warns when it
notices, and the release workflow orders the two jobs accordingly.

Install [Trunk](https://trunkrs.dev) with `cargo binstall trunk` and add the
wasm target with `rustup target add wasm32-unknown-unknown`.

For interface work, `trunk serve` in `ui/` gives you hot reload and proxies
`/api` to a server running on `:8000`.

## Layout

| | |
|---|---|
| [`api/`](./api) | The REST contract as pure serde types, shared by the server and the interface. No framework, no database — it compiles for wasm too. |
| [`server/`](./server) | The actix-web binary: handlers, storage, OIDC, telemetry, and the embedded interface. |
| [`ui/`](./ui) | The [Yew](https://yew.rs) interface, built by Trunk. Outside the Cargo workspace, because it only ever targets wasm. |
| [`tools/import-tables/`](./tools/import-tables) | The one-shot importer that moved Rex's data out of Azure Table Storage. |
| [`e2e/`](./e2e) | Playwright tests driving a real binary in a real browser. |
| [`docs/api/`](./docs/api) | OpenAPI specifications for the v1, v2, and v3 APIs. |

## Testing

```bash
cargo test --workspace          # handler, store, config, and auth tests
cd e2e && npx playwright test   # the interface, end to end
```

Two things about the test suite are worth knowing:

- **Both stores are held to one contract.** Handler tests run against the
  in-memory store because it is fast, which is only safe while it behaves
  identically to SQLite. `server/src/db/conformance.rs` runs the same
  assertions against both and fails if they ever diverge.
- **Token validation is tested against a real provider.** `server/src/testing/oidc.rs`
  serves genuine discovery, JWKS, and RS256 tokens through wiremock, so the code
  path under test is the code path that ships. There is deliberately no way to
  weaken the verifier from a test.

In debug builds the interface also serves a gallery of every control at
`/demo/controls`. It is the review surface for the control library, and it
disappears entirely from release builds.

## Deploying it

[`.deploy/`](./.deploy) holds the Kubernetes manifests. Two details matter:

- **One replica, `strategy: Recreate`.** SQLite has a single writer, and a
  rolling update would briefly run two pods against one database.
- **The database is on a `PersistentVolumeClaim`.** The container is stateless;
  everything Rex knows lives in that volume.

Secrets reach the process as environment variables and are interpolated into
`config.toml` through `${{ env.NAME }}`, so nothing sensitive is written into a
ConfigMap.

## History

Rex used to be two repositories deployed to Azure — an API on Function Apps and
a Vue interface on Blob static hosting, with Azure Table Storage underneath.
[`docs/single-repo-migration.md`](./docs/single-repo-migration.md) is the plan
that brought it here, and explains the decisions this architecture rests on.
