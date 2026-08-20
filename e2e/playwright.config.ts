import { defineConfig, devices } from "@playwright/test";

/**
 * The suite runs against one real `rex` binary with one real SQLite database.
 *
 * That means a single worker: the tests share a database, and running them
 * concurrently would have them deleting each other's ideas.
 */
export default defineConfig({
    testDir: "./tests",
    fullyParallel: false,
    workers: 1,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 1 : 0,
    reporter: process.env.CI ? [["html", { open: "never" }], ["list"]] : "list",

    use: {
        baseURL: "http://127.0.0.1:8123",
        trace: "retain-on-failure",
    },

    projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],

    webServer: {
        command: "node ./start-server.mjs",
        url: "http://127.0.0.1:8123/robots.txt",
        reuseExistingServer: !process.env.CI,
        stdout: "pipe",
        stderr: "pipe",
        timeout: 60_000,
    },
});
