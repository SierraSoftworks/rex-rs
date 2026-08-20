# End-to-end tests

These drive a real `rex` binary in a browser, against a real SQLite database.

```bash
cargo build -p rex-server
cd e2e && npm install && npx playwright install chromium && npx playwright test
```

The server the suite starts runs with **authentication disabled**, so every
request arrives as the local developer. That is deliberate: these tests are
about the application, and token validation has its own tests in
`server/src/auth/tests.rs` against a real OIDC provider.

The binary embeds `ui/dist` at compile time, so `trunk build` has to run in
`ui/` before `cargo build`, or the suite will find an error page where the
application should be.
