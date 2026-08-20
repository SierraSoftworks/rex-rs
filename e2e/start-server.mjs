// Starts the built binary against a throwaway database and configuration.
//
// The database is a fresh file per run rather than "memory", so the suite
// exercises the store that actually ships.

import { spawn } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
const binary = join(here, "..", "target", "debug", process.platform === "win32" ? "rex.exe" : "rex");

const workspace = mkdtempSync(join(tmpdir(), "rex-e2e-"));
const config = join(workspace, "config.toml");

// No `[web.auth.oidc]` section, so the server runs with authentication
// disabled and every request arrives as the local developer. Token validation
// has its own tests, against a real provider.
writeFileSync(
    config,
    [
        "[web]",
        'address = "127.0.0.1:8123"',
        `database = ${JSON.stringify(join(workspace, "rex.sqlite"))}`,
        'base_url = "http://127.0.0.1:8123"',
        "",
    ].join("\n"),
);

const server = spawn(binary, [], {
    env: { ...process.env, REX_CONFIG: config, RUST_LOG: process.env.RUST_LOG ?? "info" },
    stdio: "inherit",
});

server.on("exit", (code) => process.exit(code ?? 0));

for (const signal of ["SIGINT", "SIGTERM"]) {
    process.on(signal, () => server.kill(signal));
}
