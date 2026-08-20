# Rex: single-repo, self-contained binary migration plan

This document plans the migration of Rex from its current two-repo deployment
(`rex-rs` on Azure Function Apps + `rex-ui` on Azure Blob static hosting) to a
single repository producing one self-contained binary: an actix-web server with
an embedded Yew frontend (SSR + hydration), SQLite for storage, and standard
OIDC for authentication. The architecture follows the patterns proven in
[`automate`](https://github.com/SierraSoftworks/automate), while deliberately
retaining Rex's existing lightweight, typography-centric visual design.

## 1. Summary

| | Today | Target |
|---|---|---|
| Repos | `rex-rs` (API) + `rex-ui` (Vue 3 SPA) | `rex-rs` (workspace: `server` + `api` + `ui`) |
| Hosting | Azure Functions custom handler + Blob `$web` static site | One binary in a container (k8s Deployment + PVC) |
| Storage | Azure Table Storage (4 tables) | SQLite via `rusqlite` (bundled) + `tokio-rusqlite` |
| Frontend | Vue 3 + Element Plus + MSAL popup | Yew (SSR-rendered shell, hydrated client) |
| Auth | Hard-coded Azure AD, access tokens with `scp` scopes | Standard OIDC, config-driven, ID-token bearer (automate model) |
| Config | Scattered env vars + hard-coded constants | One `config.toml` (+ `${{ env.NAME }}` interpolation) |
| Testing | 41 handler tests against actor-based `MemoryStore` | `Services` trait: handler tests on in-memory store, conformance suite on both stores |

Two ground-truth notes discovered while surveying `automate`, since the brief
assumed otherwise:

1. **`automate` has no SSR or hydration.** Its UI is a pure CSR Yew SPA
   (`yew = { features = ["csr"] }`); the "self-contained binary" property comes
   entirely from `include_dir!` embedding Trunk's `ui/dist` output into the
   actix binary, with a `default_service` SPA fallback. SSR + hydration is
   therefore *new* architecture for Rex, not a pattern we can lift. The plan
   below builds CSR-first on automate's proven embedding story and layers SSR
   on top as its own milestone (§8), so the riskiest piece never blocks the
   rest.
2. **The database crate is `rusqlite` (feature `bundled`) wrapped by
   `tokio-rusqlite`** — there is no `tokio-sqlite`. We adopt the same stack:
   bundled SQLite means no system dependency, which is what keeps the binary
   self-contained.

## 2. Where we are today

**`rex-rs`** is already a plain actix-web 4 server; Azure Functions merely
forwards HTTP to it (`enableForwardingHttpRequest: true` in `host.json`).
Nothing in `src/` depends on the Functions payload format — the only coupling
is the `FUNCTIONS_CUSTOMHANDLER_PORT` env var (`src/main.rs`), `host.json`,
`proxies.json`, `api/function.json`, and the deploy jobs in
`.github/workflows/release.yml`. The store abstraction is 16 actix actor
messages handled by either `MemoryStore` (nested `BTreeMap`s, used by all
tests) or `TableStorage`, selected at compile time by the `table_storage`
feature. Auth is `openidconnect` 4.0 validating Azure AD ID tokens inside an
actor, with the issuer, client id, and redirect URL hard-coded
(`src/api/auth.rs`). The API surface is three coexisting versions (v1/v2/v3);
v1/v2 lean on the invariant that a user's principal id doubles as their default
collection id (`ensure_user_collection`, `src/api/utils.rs`).

**`rex-ui`** is a small Vue 3 + Vuex + Element Plus SPA (~1,400 hand-written
lines) using MSAL popup auth with a hard-coded tenant/client id. It calls only
the v3 API (plus `GET /api/v1/health` for deploy probes). Its distinctive
design lives in ~120 lines of custom CSS; everything else is stock Element
Plus. There is no dark mode, no responsive breakpoints, and no logout button.

## 3. Target architecture

### 3.1 Repository layout

The single repo is `rex-rs` (optionally renamed to `rex` later — a GitHub
rename redirects old URLs, so this can happen at any point). `rex-ui` is
archived after cutover.

```
rex-rs/
├── Cargo.toml              # workspace: members = ["server", "api", "ui"]
├── config.example.toml     # documented example config (also a schema test)
├── server/                 # bin "rex": actix-web + SQLite + OIDC + SSR + embedded assets
│   ├── build.rs            # ensures ../ui/dist exists; rerun-if-changed
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── db/             # SqliteStore, MemoryStore, migrations
│       ├── services/       # Services trait + ServicesContainer<S>
│       ├── web/            # api handlers, auth endpoints, ssr, ui asset serving
│       └── telemetry/
├── api/                    # lib "rex-api": pure serde DTOs (IdeaV1..V3, CollectionV3, ...)
│                           #   compiles natively AND for wasm; no framework deps
└── ui/                     # lib+bin "rex-ui": Yew app
    ├── index.html          # Trunk shell
    ├── Trunk.toml
    ├── styles.scss         # token definitions + @use of component styles
    └── src/
        ├── main.rs         # hydrate() on wasm
        ├── app.rs          # Route enum, App component (csr/ssr/hydration features)
        ├── auth.rs         # OIDC client tooling (ported from automate)
        ├── api.rs          # typed API client over gloo-net
        ├── components/     # each component with an adjacent .scss file
        └── views/          # each view with an adjacent .scss file
```

Differences from automate's layout, and why:

- **`ui` joins the workspace** rather than being excluded. Automate excludes it
  because the UI is wasm-only; SSR requires the `ui` crate to compile natively
  too (the server renders it), which removes the reason for exclusion. All
  browser-only code (gloo, web-sys, storage access) is gated behind
  `#[cfg(target_arch = "wasm32")]` or the `hydration` feature so the native
  build stays clean.
- **`api` follows automate's rule verbatim**: serde + chrono only, no web
  framework, no database — it is the REST contract shared by server and UI,
  replacing the duplicated TS interfaces in `rex-ui/src/api/`.

### 3.2 Runtime shape

One process, one port (default `:8000`, keeping the k8s probes and container
port stable):

- `/api/v1/**`, `/api/v2/**`, `/api/v3/**` — the JSON API (ported handlers).
- `/api/v1/auth/{metadata,token,refresh}` — OIDC broker endpoints (new, §6).
- `/healthz` → alias of `/api/v1/health` (kept for existing probes).
- Everything else — SSR-rendered app shell falling back through embedded
  static assets (`include_dir!("$CARGO_MANIFEST_DIR/../ui/dist")`), i.e.
  automate's `default_service` pattern with an SSR step in front (§8).

The Azure Functions custom-handler shim (`host.json`, `proxies.json`,
`api/function.json`, `.funcignore`) and the `INTERFACE_BACKEND_URI` reverse
proxy (`src/ui/mod.rs`) are deleted. `FUNCTIONS_CUSTOMHANDLER_PORT` becomes
`[web] address` in config. Because UI and API become same-origin, CORS —
currently a disabled TODO in `main.rs`, silently papered over by the Functions
host answering OPTIONS — stops being needed at all; we keep a config-gated
`actix-cors` layer only for the transition window while the old Blob-hosted UI
may still point at the new API.

## 4. Storage: the `Services` trait and two stores

### 4.1 From actor messages to a trait

The existing store "interface" is 16 `actor_message!` definitions plus
per-message `Handler` impls. These map 1:1 onto a plain async trait; the
`TraceMessage` machinery collapses into ordinary `#[instrument]` attributes,
and `actix` (the actor framework, not actix-web) drops out of the dependency
tree entirely.

```rust
/// Storage operations, mirroring the 16 existing actor messages.
/// Implemented by SqliteStore (production) and MemoryStore (tests/dev).
pub trait Store: Clone + Send + Sync + 'static {
    async fn get_idea(&self, collection: Id, id: Id) -> Result<Idea, Error>;
    async fn get_ideas(&self, collection: Id, filter: IdeaFilter) -> Result<Vec<Idea>, Error>;
    async fn get_random_idea(&self, collection: Id, filter: IdeaFilter) -> Result<Idea, Error>;
    async fn store_idea(&self, idea: Idea) -> Result<Idea, Error>;
    async fn remove_idea(&self, collection: Id, id: Id) -> Result<(), Error>;

    async fn get_collection(&self, id: Id, principal: Id) -> Result<Collection, Error>;
    async fn get_collections(&self, principal: Id) -> Result<Vec<Collection>, Error>;
    async fn store_collection(&self, collection: Collection) -> Result<Collection, Error>;
    async fn remove_collection(&self, id: Id, principal: Id) -> Result<(), Error>;

    async fn get_role_assignment(&self, collection: Id, principal: Id) -> Result<RoleAssignment, Error>;
    async fn get_role_assignments(&self, collection: Id) -> Result<Vec<RoleAssignment>, Error>;
    async fn store_role_assignment(&self, ra: RoleAssignment) -> Result<RoleAssignment, Error>;
    async fn remove_role_assignment(&self, collection: Id, principal: Id) -> Result<(), Error>;

    async fn get_user(&self, email_hash: Id) -> Result<User, Error>;
    async fn store_user(&self, user: User) -> Result<User, Error>;

    async fn health(&self) -> Result<Health, Error>;
}
```

Following automate (`agent/src/services/mod.rs`), the `Services` trait is the
single handle handlers receive, and the container is generic over the store so
mocking is structural rather than branchy:

```rust
pub trait Services: Clone + Send + Sync + 'static {
    type Store: Store;
    fn config(&self) -> Arc<Config>;
    fn store(&self) -> &Self::Store;
    fn session(&self) -> &Session;          // telemetry
    fn oidc(&self) -> &OidcState;           // cached discovery/JWKS (§6)
}

pub struct ServicesContainer<S: Store> { /* config, store, session, oidc */ }
pub type AppServices = ServicesContainer<SqliteStore>;

#[cfg(test)]
pub type TestServices = ServicesContainer<MemoryStore>;
```

Handlers become generic (`async fn get_idea_v3<S: Services>(services:
web::Data<S>, ...)`), registered through a generic `configure::<S>()` exactly
as automate parameterises its `TracingLogger<S>`. Rust 2024's native async fn
in traits covers this without `async-trait`; actix-web's single-threaded
workers mean we don't fight `Send` bounds on handler futures.

**`MemoryStore`** is the existing `src/store/memory.rs` (nested
`BTreeMap<u128, BTreeMap<u128, _>>` behind `RwLock`s) ported near-verbatim —
only the actor `Handler` impls change into trait methods. It stays compiled in
normal (non-test) builds too, selectable via config (`storage = "memory"`),
which gives a zero-setup dev mode and keeps it honest.

### 4.2 SQLite schema

`SqliteDatabase` follows automate (`agent/src/db/sqlite.rs`): a single shared
`tokio_rusqlite::Connection` in an `Arc`, configured with `busy_timeout(5s)`,
`PRAGMA journal_mode = WAL` (skipped for `:memory:`), and `synchronous =
NORMAL` only when WAL took. Migrations are a `const MIGRATIONS: &[&str]` array
applied transactionally with versions tracked in a `migrations` table; we also
adopt `open_in_memory_at_migration(version)` for migration tests. Unlike
automate's KV-of-JSON design, Rex's entities are small, stable, and relational
— so we use real tables:

```sql
CREATE TABLE users (
    principal_id TEXT PRIMARY KEY,          -- 32-char zero-padded lowercase hex
    email_hash   TEXT NOT NULL UNIQUE,      -- md5(lowercase(trimmed email)), hex
    first_name   TEXT NOT NULL
);

CREATE TABLE collections (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE role_assignments (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    principal_id  TEXT NOT NULL,
    role          TEXT NOT NULL CHECK (role IN ('Owner','Contributor','Viewer')),
    PRIMARY KEY (collection_id, principal_id)
);

CREATE TABLE ideas (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    id            TEXT NOT NULL,
    name          TEXT NOT NULL,
    description   TEXT NOT NULL,
    completed     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (collection_id, id)
);

CREATE TABLE idea_tags (
    collection_id TEXT NOT NULL,
    idea_id       TEXT NOT NULL,
    tag           TEXT NOT NULL,
    PRIMARY KEY (collection_id, idea_id, tag),
    FOREIGN KEY (collection_id, idea_id)
        REFERENCES ideas(collection_id, id) ON DELETE CASCADE
);
CREATE INDEX idea_tags_by_tag ON idea_tags(collection_id, tag);
```

Deliberate decisions baked into this schema:

- **IDs stay 32-char hex `TEXT`.** `u128` in memory, `format!("{:0>32x}")` at
  the boundary — identical to today's wire format and Table Storage keys, so
  the data migration is a straight copy and existing bookmarks/ids survive.
- **The `principal_id == default collection id` invariant is preserved** —
  `ensure_user_collection` keeps working unchanged, which is what keeps the
  v1/v2 API alive.
- **Tags become a junction table.** Today they're a comma-joined string (with
  a leading comma) filtered via OData `contains()` — which false-positives on
  substrings and needs a client-side re-filter. `idea_tags` fixes the
  correctness bug and makes tag filtering an indexed join. Random selection
  becomes `ORDER BY RANDOM() LIMIT 1` instead of load-everything-and-pick.
- **Collections are normalised.** Table Storage duplicates a shared
  collection's row into every member's partition (copied at role-assignment
  time), so a post-share rename by the owner silently diverges per member.
  One `collections` row + the `role_assignments` join fixes that; the visible
  behaviour change (renames now reach all members) is an improvement we accept
  deliberately. `GetCollections(principal)` becomes a join instead of a
  partition scan; the `CollectionV3.userId` field is populated with the
  requesting principal, matching what the UI already assumes.
- **Foreign keys ON** (unlike automate, which has no relations to enforce);
  `PRAGMA foreign_keys = ON` per connection.
- **Health does real work**: `SELECT 1` through the connection rather than the
  current unconditional `ok: true`.

### 4.3 Keeping both stores honest

Handler unit tests run against `MemoryStore` — cheap, no SQLite in the loop,
exactly what the current `test_state!`/`test_request!` macros do today (those
macros port across with the actor `send` swapped for trait calls). To avoid
automate's stated failure mode — "the thing under test is not the thing that
ships" — a **conformance suite** exercises every `Store` method through a
generic `fn store_contract<S: Store>(make: impl Fn() -> S)` and runs it twice:
once on `MemoryStore`, once on `SqliteStore::open_in_memory()`. Any behavioural
drift (ordering, not-found errors, upsert semantics, random-with-filter edge
cases) fails both ways.

## 5. API surface

All v1/v2/v3 routes port mechanically: the handlers are thin, the extractor
structs (`IdFilter`, `CollectionFilter`, `QueryFilter`, …), the
`json_responder!` macro (201 + `Location` on POST), and the `APIError` shape
(`{code, error, message}`) carry over unchanged — the UI's error toasts depend
on that shape. The two Azure `From` impls on `APIError` are replaced by one
`From<tokio_rusqlite::Error>`. `GET /api/v1/auth`, documented in the OpenAPI
specs but never implemented, is superseded by the real auth endpoints (§6).

Recommendation: **keep v1 and v2**. They cost almost nothing (thin wrappers
over the same store ops), preserve the healthcheck contract, and any scripted
consumers keep working through the auth transition. If we later confirm nobody
uses them, deleting them is a one-commit cleanup.

## 6. Authentication: standard OIDC

### 6.1 Model

We adopt automate's flow wholesale: the browser runs the Authorization Code
flow in a popup, the server performs the confidential code exchange with its
client secret, and the **ID token is the bearer**, held in `sessionStorage`
and sent as `Authorization: Bearer`. No cookies, so no CSRF surface. The
server exposes three endpoints (`agent/src/web/api/auth.rs` in automate):

- `GET  /api/v1/auth/metadata` → `{authorization_endpoint, client_id, scopes}`
  from cached discovery, so the browser never reads discovery cross-origin.
- `POST /api/v1/auth/token` → authorization-code exchange.
- `POST /api/v1/auth/refresh` → refresh grant (hence `offline_access` in the
  default scopes).

This stays compatible with the current Azure AD tenant — Azure AD is a
standard OIDC provider via its v2.0 endpoints — so cutover does not force an
IdP change; it just becomes *configuration* (and any other OIDC provider works
identically).

### 6.2 What we reuse from `rex-rs`, and what changes

The provider-agnostic machinery in `src/api/auth.rs` carries over: the
`AuthToken` type with its claim accessors, the `FromRequest` extractor (any
handler taking `token: AuthToken` is authenticated), the 401/403 error shapes,
and the `require_role!` macro. What changes:

| Today (Azure-specific) | Target (standard OIDC) |
|---|---|
| `oid` claim → principal id | configurable `username_claim` (default `sub`), hashed/parsed into the same `u128` id space |
| `unique_name` → email | `email` claim (standard scope) |
| `scp` scope checks (`Ideas.Read`, …) | **retired** — see below |
| `roles` claim (`Administrator`/`User`) | optional `user_acl` / `admin_acl` config gates (filt-rs, automate-style), evaluated over `claims.*` |
| Hard-coded issuer/client-id/redirect | `[web.auth.oidc]` config (§9) |
| `OidcActor` (503 until discovery completes; mailbox hop per request) | cached discovery + JWKS in an `ArcSwap`/`tokio::sync::OnceCell` with TTL refresh and a refetch-once path on unknown `kid` (automate's `oidc.rs` pattern) |

**Retiring scope checks.** The `Ideas.Read`-style scopes are Azure AD
delegated permissions on an access token; a standard OIDC ID token has no
`scp`. They were only ever coarse gates in front of the real authorization —
the per-collection `RoleAssignment` (Owner/Contributor/Viewer) checks, which
live in the handlers and store and **carry over unchanged**. The `user_acl`
gate (deny-by-default, evaluated in middleware) replaces the `require_role!`
"is this a known user" function. Token validation itself follows automate's
hardening: reject HMAC algorithms outright (algorithm-confusion defence),
validate `aud` = client id, `iss` from discovery, `exp`/`nbf`.

Note on `sub` vs today's ids: existing users' principal ids are Azure AD
`oid` GUIDs. Azure AD's v2.0 `oid` claim is still present and stable, so the
default config for the Sierra Softworks deployment sets
`username_claim = "oid"`, keeping every existing user attached to their data.
Fresh deployments default to `sub`. Because the claim value is parsed through
the same dashes-stripped hex path as today, no data rewrite is needed.

### 6.3 Client-side auth tooling (ported from automate's `ui/src/auth.rs`)

- `begin_login()`: random 24-byte `state` via `crypto.getRandomValues`, fetch
  `/api/v1/auth/metadata`, open the provider in a popup, poll a `localStorage`
  handoff slot (popups don't share `sessionStorage`).
- `/auth/callback` route: verify `state`, POST the code to
  `/api/v1/auth/token`, hand tokens to the opener and close.
- `sessionStorage` keys `rex.token` / `rex.refresh` / `rex.oidc.state`;
  `localStorage` `rex.oidc.popup_result` for the handoff.
- **Single-flight refresh** (`SingleFlight` over `futures::future::Shared`) so
  N concurrent 401s collapse into one refresh — automate learned this the hard
  way; we take it from day one.
- `AuthStatus::{Loading, Disabled, SignedIn, NeedsLogin, Forbidden, Error}`
  resolved by probing `GET /api/v1/auth/me` (new, trivial endpoint returning
  the validated principal) rather than parsing the token client-side; exposed
  via `ContextProvider<AuthHandle>`; a `Protected` wrapper component mounts
  children only once access is granted.
- The API client attaches the bearer on **every** request in one place
  (automate's rule: a page that forgot the header would silently misbehave).
- **We add a logout button** — the current UI defines `ACT_LOGOUT` but never
  renders a way to trigger it.

UX note: the current UI is already popup-based (MSAL `loginPopup`), so this is
not a flow regression — but we keep automate's discipline of only opening the
popup from an explicit click, fixing rex-ui's current footgun where any API
call by a signed-out user triggers a popup that blockers eat.

## 7. UI: Yew port that keeps Rex's face

### 7.1 Design language

The entire custom design is ~120 lines across `index.css`, `App.vue`,
`IdeaDisplay.vue`, and `TagEditor.vue`. It ports verbatim into SCSS tokens:

```scss
// ui/styles.scss — tokens
$font-body: 'Raleway', 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
$font-display: 'Montserrat';        // headings + header chrome
$color-ink: #000;                   // wordmark, strong text
$color-text: #444;                  // menu/body chrome
$color-surface: #fff;
$color-hover: #eee;
$color-hairline: #f6f6f6;           // the near-invisible header border
$color-code: #B21D12;               // the single chromatic accent
$color-primary: #409EFF;            // Element Plus blue, kept for familiarity
$opacity-deemphasis: 0.3;           // the "/" separator, the hanging quote
$header-height: 65px;
```

Non-negotiables to reproduce exactly:

- `h1` at `font-weight: 100`, `2.5em`; all headings Montserrat weight 300 —
  the thin display type *is* the brand.
- The **hanging U+201C quote glyph**: 40px, `opacity: 0.3`, absolutely offset
  `-40px` into the left margin of the idea title. This is the product's
  signature; it ships in `idea_display.scss` pixel-identical.
- The `.fill`/`.center` absolute-centering primitive: one shrink-wrapped block
  on an otherwise empty canvas. The emptiness is the design — no max-width
  containers, no cards.
- Fixed full-viewport shell: only the content pane scrolls; header is a
  hairline-bordered 65px bar with full-height hover targets washing to `#eee`
  over `0.25s`, `REX / collection` breadcrumb with the lowercase collection
  name.
- Inline code in `#B21D12` with all chrome stripped; **no highlight.js theme**
  (fenced blocks are currently unthemed — reproducing the look means not
  adding one).
- Light-only. No dark mode exists today and we don't add one during migration.

What we do *not* port: Element Plus. The dozen stock components rex-ui uses
(button, input, form, table, tag, select, avatar, notification, tooltip,
loading) are rebuilt as our own controls (§7.3), styled minimally in the same
typographic register — thin weights, hairlines, generous whitespace — with
`$color-primary` kept at Element's blue so the app still reads as Rex. This is
where automate's design language explicitly does **not** come along: no brand
teal, no card shadows, no dense admin chrome.

### 7.2 SCSS organisation

Per the brief: SASS, with component/view styles adjacent to their Rust files
rather than monolithic (a deliberate departure from automate's single
2,953-line `styles.scss`):

```
ui/src/components/idea_display.rs
ui/src/components/idea_display.scss
ui/src/views/home.rs
ui/src/views/home.scss
```

Trunk compiles one SCSS entry (`<link data-trunk rel="scss"
href="styles.scss">`), so the root `styles.scss` holds tokens/reset/layout and
explicitly `@use`s every adjacent file (Sass has no globbing). The convention
is enforced socially by the controls gallery (§7.3) — an unwired stylesheet is
immediately visible there — plus a small unit test in the server crate that
asserts every `.scss` under `ui/src/` is referenced from `styles.scss`, so an
omission fails CI rather than shipping unstyled.

Class naming stays BEM-ish (`.idea__title`, `.header__name`) to match the
existing CSS being ported.

### 7.3 Controls library + debug-only gallery

Controls (initial inventory, derived from what Element Plus provides today):
`Button`, `ButtonGroup`, `IconButton`, `TextInput`, `TextArea`, `Select`,
`Tag`, `TagEditor`, `Form`/`FormField`, `Table`, `Avatar`, `Notification`
(toast host + context), `Tooltip`, `Spinner`/loading overlay, `MarkdownView`
(pulldown-cmark, replacing markdown-it), `DateDisplay` (dropped — dead code in
rex-ui).

The gallery replicates automate's triple `#[cfg(debug_assertions)]` gating
exactly (route variants, switch arms, module + re-export), mounted at
`/demo/controls` and `/demo/controls/:control`, with specimens registered in a
`CONTROLS` slice and the standing convention: *add a specimen when you add or
change a control*. Debug builds only — the release binary contains none of it.
Automate's demo-fixtures pattern (an in-memory `thread_local!` store of
`Vec`s behind a `demo!` macro in the API client) is also worth carrying so the
gallery and views render without a backend; note the pleasant symmetry — the
UI's mock is the HashMap/Vec store, while the server's is `MemoryStore`.

### 7.4 Views and routes

Routes keep today's paths (bookmarks survive):

| Route | View | Notes |
|---|---|---|
| `/` | Home | random idea from default collection; the core loop (Done / Undo / Delete / Next) |
| `/collection/:cid` | Home | same view, scoped |
| `/collection/:cid/idea/:iid` | Idea | fix the currently-unreachable delete button |
| `/collection/:cid/new` | New idea | form: name, description, TagEditor |
| `/collections` | Collections | list + Open / New Idea / Manage |
| `/collections/new` | New collection | |
| `/collection/:cid/manage` | Manage | ideas table + members table; define the missing `.success-row` style for completed ideas |
| `/collection/:cid/invite` | Invite | debounced email→md5 lookup, Gravatar preview, role select |
| `/auth/callback` | (auth) | new — popup landing |
| `/demo/controls[/:control]` | Gallery | debug builds only |

Behavioural fixes we make deliberately rather than porting bugs: consistent
error toasts on all mutations (today most `.catch`es are missing, so failures
vanish), a visible "no such user" state on invite, a logout affordance, and
`<Switch>` keyed by route (matching `:key="$route.fullPath"` remount
semantics). Gravatar stays (it's part of the look); the hardcoded Eleme CDN
fallback avatar is replaced with a local asset.

## 8. SSR + hydration

This is the one genuinely new piece of architecture. Design:

- The `ui` crate exposes cargo features `csr`, `ssr`, `hydration`
  (yew 0.23 supports all three). `trunk serve` dev builds use `csr`; the
  shipped wasm bundle is built with `hydration`; the server depends on
  `rex-ui` with `ssr`.
- **Server side**: the SSR handler renders `<ServerApp>` — the same component
  tree wrapped in a `yew_router` memory history primed with the request path —
  via `yew::ServerRenderer`, then splices the result into the *built*
  `ui/dist/index.html` shell (which carries Trunk's hashed script/link tags)
  at a body marker. Static assets still come from `include_dir!`; only
  navigational paths hit the renderer. We add the cache headers automate
  forgot: immutable + long max-age for hashed assets, `no-cache` for the
  document.
- **Client side**: `main()` calls `yew::Renderer::hydrate()` under
  `#[cfg(target_arch = "wasm32")]`, attaching to the SSR output.
- **Scope discipline — SSR renders the anonymous shell only.** Bearer tokens
  live in `sessionStorage`, so the server *cannot* see who the user is; every
  authenticated view SSRs as its logged-out/loading skeleton and CSR takes
  over after hydration + auth resolution. That means: no `use_prepared_state`
  data plumbing in v1, no server-side API calls during render, and components
  must simply render sensibly in the `AuthStatus::Loading` state. What SSR
  buys us at this scope: instant first paint of the app chrome and login
  screen, correct `<title>`/meta, and a working page when wasm is slow to
  load. What it doesn't buy: pre-rendered user data — a fine trade for an app
  that is 100% behind login.
- **Sequencing**: milestones M1–M4 build and ship CSR-only on automate's
  embedding pattern (`default_service` → `index.html`). M5 swaps the fallback
  for the SSR handler. If SSR hits an unexpected wall (a wasm-only dependency
  that can't be gated, hydration mismatches), the CSR fallback is already the
  working production configuration — SSR never blocks cutover.

The main engineering cost is keeping the `ui` crate dual-target: every use of
`web_sys`/`gloo` (storage, popups, timers) sits behind `#[cfg(target_arch =
"wasm32")]` with inert native stubs. The `Protected`/auth components make this
tractable since data access already funnels through `api.rs` and `auth.rs` —
two files to gate, not thirty.

## 9. Configuration

One `config.toml`, loaded automate-style: `dotenvy` first, then `${{ env.NAME
}}` interpolation, then `toml::from_str` into structs with
`#[serde(deny_unknown_fields)]` everywhere. Two patterns adopted verbatim
because they're cheap and permanent: unknown keys are startup errors, and
`config.example.toml` is parsed in a unit test so documentation drift fails CI.

```toml
# config.example.toml
[web]
address = "0.0.0.0:8000"
database = "rex.sqlite"            # or "memory" for a throwaway dev instance
# base_url = "https://rex.sierrasoftworks.com"   # for absolute redirect URIs

[web.auth]
# Deny-by-default ACLs evaluated over validated token claims (claims.*),
# client_ip, method, and path. user_acl gates sign-in; admin_acl gates
# any future administrative surface.
# user_acl = 'claims.email matches ".*"'
# admin_acl = 'false'

[web.auth.oidc]
endpoint = "https://login.microsoftonline.com/a26571f1-22b3-4756-ac7b-39ca684fab48/v2.0"
client_id = "${{ env.REX_OIDC_CLIENT_ID }}"
client_secret = "${{ env.REX_OIDC_CLIENT_SECRET }}"
scopes = ["openid", "profile", "email", "offline_access"]
username_claim = "oid"             # keeps existing Azure AD principals stable;
                                   # new deployments use the default, "sub"

[telemetry]
# sentry_dsn = "${{ env.SENTRY_DSN }}"
# otlp_endpoint = "https://api.honeycomb.io:443"
# otlp_headers = { x-honeycomb-team = "${{ env.HONEYCOMB_KEY }}" }
```

This absorbs everything currently hard-coded: the OIDC issuer/client-id
(`src/api/auth.rs`), the Sentry DSN, OTLP endpoint, and Medama endpoint
(`src/telemetry/mod.rs`), and the listen port. `TABLE_STORAGE_CONNECTION_STRING`
dies with Table Storage.

## 10. Observability

`tracing-batteries` stays (Sentry + OpenTelemetry + analytics), now fed from
config instead of constants. The hand-rolled `TracingLogger` middleware is
replaced by automate's version, which we want anyway for its **redaction
lists**: rex-rs's current middleware records `http.headers` wholesale —
including `Authorization` — into spans; automate's redacts
authorization/cookie headers and `code`/`state`/`token` query params by name.
Store spans move from `db.system = "TABLESTORAGE"` to `db.system = "sqlite"`
with `db.operation` per method via `#[instrument]` on the trait impls. The
`TraceMessage` actor-propagation layer is deleted — with no actor boundary,
ordinary span parenting does the job.

## 11. Testing strategy

- **Handler tests** (the existing 41 port across): `test_state!` seeds a
  `ServicesContainer<MemoryStore>`, `test_request!` drives the real app
  factory. Fast, no SQLite, no network — the brief's motivating case.
- **Store conformance suite** (§4.3): both stores, same assertions.
- **Migration tests**: `open_in_memory_at_migration(n)` + seed + migrate-rest.
- **Auth tests**: automate's `testing/oidc.rs` is adopted wholesale — a
  wiremock OIDC provider serving real discovery, real JWKS, real RS256 tokens
  — so token validation is tested against the code path that ships, with *no
  test-only branches in shipping code*. This deletes rex-rs's current
  `#[cfg(test)] insecure_disable_signature_check` weakening, which is exactly
  the pattern automate's docs warn about.
- **UI**: controls get rendered-output tests where valuable; the gallery is
  the primary review surface. **E2E**: a Playwright suite on automate's
  template (single worker, one SQLite behind one real binary, readiness probed
  via `/robots.txt`, `TrunkApplicationStarted` event instead of `networkidle`)
  covering the core loop: login (against the wiremock IdP or a disabled-auth
  config), create idea, random cycle, complete, manage, invite.

## 12. Build, CI, packaging

Adopted from automate's `rust.yml` with the artifact handoff made explicit:

1. `test` — `cargo test` (server + api) with coverage → codecov.
2. `ui` — Trunk via `cargo binstall`; build **twice**: debug dist (gallery +
   fixtures compiled in) uploaded for e2e, `--release` dist uploaded for
   packaging.
3. `e2e` — download debug dist into `ui/dist`, `cargo build -p rex-server`,
   Playwright.
4. `build` — target matrix (linux musl x64/arm64, macOS x64/arm64, windows),
   each downloading the release dist into `ui/dist` **before** `cargo build`,
   producing truly self-contained binaries (`rusqlite/bundled` + rustls; the
   `openssl-vendored` escape hatch only if a dependency forces OpenSSL).
5. `docker-build`/`docker-publish` — automate's package-prebuilt-binary
   Dockerfile (ubuntu + ca-certificates, `ADD rex`), multi-arch manifest.

`server/build.rs` reproduces automate's `create_dir_all(../ui/dist)` +
`cargo:rerun-if-changed=../ui/dist` glue, including the war-story comment about
`.gitkeep` — and we inherit the known sharp edge that a binary built before
`trunk build` serves a 500 shell (mitigated by the build.rs warning and CI
ordering; documented in the README).

Deployment target is the existing k8s manifests (`.deploy/`), which already
run the binary standalone on port 8000 — the delta is: `TABLE_STORAGE_CONNECTION_STRING`
secret → a PVC mounted for `rex.sqlite` + a ConfigMap/secret for `config.toml`,
and the Deployment moves to `strategy: Recreate` (one SQLite writer; no
overlapping pods during rollout). Probes stay on `/api/v1/health`, which now
actually checks the database.

## 13. Data migration from Table Storage

A one-shot importer, kept out of the shipping binary (a `tools/import-tables`
bin in the workspace, or a `rex import-tables` subcommand behind a non-default
`azure-import` feature — recommendation: the former; it's used once):

1. Read all four tables (`ideas`, `collections`, `roleassignments`, `users`)
   via `azure_data_tables` using the existing connection string.
2. Transform:
   - keys: partition/row hex strings → same hex strings (no re-encoding);
   - tags: split the comma-joined string, drop the historical leading comma,
     trim empties → `idea_tags` rows;
   - collections: **dedup** the per-member duplicates into one row keyed by
     collection id. Where duplicated names diverged (the rename bug), keep the
     Owner's copy and log the losers;
   - role assignments and users: straight copies.
3. Load into a fresh SQLite file through the *real* `SqliteStore` migrations
   (so the importer can't invent its own schema), inside one transaction per
   table.
4. Verify: row counts per table, spot-check a sample of ideas round-tripped
   through `GET /api/v3/...` against the live API, and a full dump-diff of ids.

The importer is idempotent (re-runnable against a fresh file) so cutover can
rehearse it any number of times against production data before the real one.

## 14. Cutover & decommissioning

1. **Staging first**: deploy the binary to k8s staging with a PVC; run the
   importer against a production snapshot; point `rex-staging.sierrasoftworks.com`
   at it (replacing the Blob static site + staging Function App). Soak: auth
   flows on real Azure AD via OIDC, the core loop, manage/invite.
2. **App registration**: add the SPA popup redirect URI
   (`https://rex.sierrasoftworks.com/auth/callback`), issue a client secret
   for the confidential exchange; the old delegated-permission scopes become
   unused and are removed after cutover.
3. **Production cutover** (low-traffic window): scale the Function App's
   writes off (announce/brief freeze), run the final import, flip DNS/ingress
   for `rex.sierrasoftworks.com` to the k8s service, verify health + a smoke
   pass.
4. **Rollback window**: keep the Function App and Table Storage untouched for
   ~2 weeks — rollback is a DNS flip back (accepting loss of writes made to
   SQLite in the interim, or re-running a reverse export if it matters).
5. **Decommission**: Function Apps (staging + live), the `$web` static-site
   storage accounts, the old Azure deploy jobs in both repos' workflows;
   archive `rex-ui` with a pointer to the new repo; update the README, the
   OpenAPI specs (auth section), and the Sentry project settings (one project,
   one DSN, from config).

## 15. Milestones

Each milestone is independently mergeable and leaves `main` shippable.

| # | Milestone | Contents | Exit criteria |
|---|---|---|---|
| M0 | Workspace scaffold | `server`/`api`/`ui` workspace; config system (+example-file test); telemetry from config; Functions shim deleted; port/CORS from config | binary serves the existing API on `:8000` from `MemoryStore`; CI green without Azure deploy jobs |
| M1 | Storage | `Store` + `Services` traits; `MemoryStore` port; `SqliteStore` + migrations; conformance suite; handlers de-actored | all 41 existing tests pass on the trait; conformance green on both stores; health does `SELECT 1` |
| M2 | Auth | OIDC config; cached discovery/JWKS validation; `/auth/{metadata,token,refresh,me}`; `user_acl`; scope-check retirement; wiremock IdP tests | sign-in against real Azure AD via standard OIDC on a dev instance; no test-only validation weakening left |
| M3 | UI foundation | Trunk + Yew skeleton; SCSS tokens + ported global styles; controls library; `#[cfg(debug_assertions)]` gallery + demo fixtures; embedded-asset serving with cache headers | gallery renders every control pixel-faithful to the current design; binary serves the SPA |
| M4 | UI features | client auth tooling; API client; all 8 views; error toasts; logout | feature parity with rex-ui (minus its bugs) against the M1/M2 server; e2e suite green |
| M5 | SSR + hydration | dual-target `ui` crate; `ServerRenderer` + shell splicing; `hydrate()` entry | anonymous shell SSRs; hydration attaches without mismatch warnings; CSR fallback removable |
| M6 | Migration + cutover | importer; staging rehearsal; CI release matrix + Docker; k8s manifests (PVC, Recreate) | staging serving imported production data end-to-end |
| M7 | Go live | production cutover, rollback window, decommission, archive `rex-ui` | old infrastructure deleted; docs updated |

M3/M4 can proceed in parallel with M1/M2 after M0 (the UI develops against
demo fixtures until the server is ready).

## 16. Risks

- **SSR dual-target compilation** is the highest-variance item: a transitive
  wasm-only dependency or a hydration mismatch could stall it. Mitigated by
  sequencing (CSR-only is the working state from M3) and by confining
  browser APIs to `auth.rs`/`api.rs`.
- **SQLite single-writer**: fine at Rex's scale with WAL + busy_timeout, but it
  forces single-replica (`Recreate`) deployments. Accepted; automate runs the
  same way.
- **Auth cutover**: users mid-session during DNS flip get logged out (token
  audience changes from the API app-id URI to the client id). Accepted — it's
  a one-time re-login.
- **Data fidelity**: the collection dedup makes a choice where names diverged;
  logged and manually reviewable before go-live. Tag splitting is lossy only
  for tags containing commas, which the current UI can't create.
- **Design fidelity**: rebuilding Element Plus components risks visual drift.
  Mitigated by the gallery (side-by-side review against the live app) and by
  keeping the Element blue and default control geometry as the starting point.

## 17. Open questions

1. **Keep v1/v2 APIs?** Recommendation: yes (near-zero cost). Confirm no
   external consumers depend on the retired `scp` scopes before deleting the
   OpenAPI scope documentation.
2. **IdP after migration**: stay on Azure AD (zero user-visible change,
   `username_claim = "oid"`), or take the opportunity to move to another OIDC
   provider? The design is provider-neutral either way.
3. **Repo rename** `rex-rs` → `rex` once it stops being "the Rust half"?
4. **Collection rename semantics**: confirm that shared-collection renames
   becoming visible to all members is desired (it falls out of normalisation).
5. **Multi-tenancy**: automate's `TenantDb` structural isolation is *not*
   carried over — Rex's per-collection roles are its authz model. Flagging in
   case a tenant boundary is ever wanted; it would change the schema.
