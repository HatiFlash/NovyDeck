import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  timeout: 60_000,
  expect: { timeout: 10_000 },
  use: { baseURL: "http://127.0.0.1:8766", trace: "retain-on-failure" },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: { command: "python3 -m http.server 8766 --directory out --bind 127.0.0.1", url: "http://127.0.0.1:8766", reuseExistingServer: true, timeout: 15_000 },
});
