# Rex
**Keep track of your ideas for things to do and get handed a random one on demand**

Rex keeps a list of the things you've been meaning to do and, when you can't decide,
picks one for you. Ideas live in collections which you can share with other people, and
each one carries a Markdown description and a set of tags you can filter on.

Rex ships as a single self-contained binary: an actix-web server with the web interface
compiled to WebAssembly and embedded in the executable, SQLite compiled in alongside it,
and authentication against any standard OpenID Connect provider.

## Installation
Run the Docker image (published for `linux/amd64` and `linux/arm64`):

```sh
docker run -p 8000:8000 \
  -v $(pwd)/config.toml:/config/config.toml:ro \
  -v rex-data:/data \
  ghcr.io/sierrasoftworks/rex-rs:latest
```

Pre-compiled binaries for Linux, macOS, and Windows are available from the
[GitHub releases](https://github.com/SierraSoftworks/rex-rs/releases) page.

## Highlights
- **One binary, one port.** The web interface, the API, and the database all live in the
  executable. There is nothing else to deploy and nothing else to run.
- **Collections and sharing.** Share a collection with someone by email address and give
  them an Owner, Contributor, or Viewer role. Permissions are enforced per collection.
- **Standard OIDC.** Point Rex at any OpenID Connect provider through `config.toml`. The
  browser runs the authorization code flow in a popup, the server performs the
  confidential exchange, and the resulting ID token is the bearer.
- **Markdown and tags.** Descriptions render as CommonMark. Tags are indexed, so
  `?tag=cooking` is a join rather than a scan.
- **Three API versions.** v1, v2, and v3 are all served.
- **Native OpenTelemetry.** Export traces to any OTLP endpoint, with Sentry and analytics
  alongside it.

## Quick start
```bash
cargo run -p rex-server
```

With no `config.toml` present, Rex listens on `:8000`, keeps its data in `./rex.sqlite`,
and runs with authentication disabled. Copy
[`config.example.toml`](./config.example.toml) to `config.toml` to change any of that —
every option is documented there. Set `REX_CONFIG` to load it from somewhere else.

> ⚠️ Without a `[web.auth.oidc]` section, every request is treated as the same local
> developer. Use it while developing, never on a deployment anyone else can reach.

## Configuration
A configuration file is composed of three sections:

```toml
[web]
address = "0.0.0.0:8000"                       # The socket to listen on.
database = "rex.sqlite"                        # Or "memory" for a throwaway store.
base_url = "https://rex.example.com"           # Used to build the OIDC redirect URI.
cors_origins = []                              # Extra origins allowed to call the API.

[web.auth]                                     # Optional access gates (see Authentication).
[web.auth.oidc]                                # Omit to run without authentication.
[telemetry]                                    # Optional Sentry, OTLP, and analytics.
```

Values of the form `${{ env.NAME }}` are replaced with that environment variable before
the file is parsed. An unset variable stops the server, as does an unrecognised key.

## Authentication
Configure a provider under `[web.auth.oidc]`:

```toml
[web.auth.oidc]
endpoint = "https://auth.example.com"
client_id = "rex"
client_secret = "${{ env.REX_OIDC_CLIENT_SECRET }}"
scopes = ["openid", "profile", "email", "offline_access"]
username_claim = "sub"    # Which claim carries the principal id.
email_claim = "email"     # Which claim carries the email address.
```

Register `{base_url}/auth/callback` as a redirect URI with your provider. `offline_access`
gets you a refresh token, so a session outlives the ID token.

Rex validates the ID token on every request: asymmetric signatures only, audience matched
against `client_id`, issuer from discovery. Discovery documents and JWKS are cached, and an
unrecognised signing key forces a refetch.

Who may sign in at all is decided by `user_acl`, a [`filt-rs`](https://docs.rs/filt-rs)
expression over the validated claims, `client_ip`, `method`, and `path`:

```toml
[web.auth]
user_acl = 'claims.email endswith "@example.com"'
```

Omitting `user_acl` admits any principal holding a valid token from the configured
provider. What a signed-in user can then *do* is decided by their role on each collection.

## API
Three versions are served, all under `/api`. v3 is the current one and the only one that
knows about collections.

| Route | |
|---|---|
| `GET /api/v3/collections` | Collections you have a role on. |
| `GET /api/v3/collection/{id}/ideas` | Ideas in a collection, filtered by `?tag=` and `?complete=`. |
| `GET /api/v3/collection/{id}/idea/random` | One idea, chosen at random, honouring the same filters. |
| `PUT /api/v3/collection/{id}/user/{user}` | Grant someone a role. |
| `GET /api/v3/user/{email_hash}` | Look someone up by the MD5 of their email address. |
| `GET /api/v1/auth/metadata` | How to sign in to this instance. |
| `POST /api/v1/auth/token`, `/refresh` | The server-side halves of the OIDC flow. |
| `GET /api/v1/auth/me` | The identity Rex resolved from your token. |
| `GET /api/v1/health`, `/healthz` | Liveness and readiness, including a real database query. |

Full OpenAPI specifications live in [`docs/api/`](./docs/api).

Errors are `{code, error, message}` throughout. Roles are `Owner`, `Contributor`, and
`Viewer`; a collection you hold no role on reads as one that does not exist.

## Development
The server embeds `ui/dist` at compile time, so the interface is built first:

```bash
cd ui && trunk build && cd ..
cargo build -p rex-server
```

Build the server without doing that and you get a binary that serves a plain error page in
place of the interface. Install [Trunk](https://trunkrs.dev) with `cargo binstall trunk`
and add the wasm target with `rustup target add wasm32-unknown-unknown`.

For interface work, `trunk serve` in `ui/` gives you hot reload and proxies `/api` to a
server on `:8000`. Debug builds also serve a gallery of every control at `/demo/controls`;
add a specimen when you add a control.

```bash
cargo test --workspace                     # handlers, stores, config, and auth
cargo test --manifest-path ui/Cargo.toml   # the interface's pure-Rust parts
cd e2e && npx playwright test              # the interface, end to end
```

Handler tests run against an in-memory store.
[`server/src/db/conformance.rs`](./server/src/db/conformance.rs) runs the same assertions
against that store and against SQLite. Token validation is tested against a wiremock
provider serving real discovery, JWKS, and RS256 tokens.

| | |
|---|---|
| [`api/`](./api) | The REST contract as pure serde types, shared by the server and the interface. Compiles for wasm. |
| [`server/`](./server) | The actix-web binary: handlers, storage, OIDC, telemetry, embedded assets. |
| [`ui/`](./ui) | The [Yew](https://yew.rs) interface, built by Trunk. Outside the Cargo workspace — it only targets wasm. |
| [`tools/import-tables/`](./tools/import-tables) | The one-shot importer that moved Rex off Azure Table Storage. |
| [`e2e/`](./e2e) | Playwright tests driving a real binary in a real browser. |

## Deployment
[`.deploy/`](./.deploy) holds Kubernetes manifests: a Deployment, Service, ConfigMap, and a
PersistentVolumeClaim for the database.

SQLite has a single writer, so the Deployment runs one replica with `strategy: Recreate`.
The container is stateless; everything Rex knows lives in the volume.

Secrets reach the container as environment variables and are interpolated into
`config.toml` at startup, keeping them out of the ConfigMap.

## Learn more
- **API specifications:** [`docs/api/`](./docs/api)
- **Downloads:** <https://github.com/SierraSoftworks/rex-rs/releases>
- **Report an issue:** <https://github.com/SierraSoftworks/rex-rs/issues/new>
- **How Rex got here:** [`docs/single-repo-migration.md`](./docs/single-repo-migration.md)
  is the plan behind the move from two Azure-hosted repositories to this one.
