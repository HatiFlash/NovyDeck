import { chromium } from "playwright";
import { writeFile } from "node:fs/promises";

const browser = await chromium.launch({ headless: true, args: ["--enable-unsafe-swiftshader"] });
const page = await browser.newPage({ viewport: { width: 1365, height: 768 }, deviceScaleFactor: 1 });
const cdp = await page.context().newCDPSession(page);
await cdp.send("Performance.enable");
await cdp.send("Emulation.setCPUThrottlingRate", { rate: 4 });
await page.addInitScript(() => {
  window.__launchVitals = { lcp: 0, cls: 0 };
  new PerformanceObserver(list => {
    for (const entry of list.getEntries()) window.__launchVitals.lcp = Math.max(window.__launchVitals.lcp, entry.startTime);
  }).observe({ type: "largest-contentful-paint", buffered: true });
  new PerformanceObserver(list => {
    for (const entry of list.getEntries()) if (!entry.hadRecentInput) window.__launchVitals.cls += entry.value;
  }).observe({ type: "layout-shift", buffered: true });
});
const errors = [];
page.on("pageerror", error => errors.push(error.message));
await page.goto("http://127.0.0.1:8766/", { waitUntil: "networkidle" });
await page.evaluate(() => document.fonts.ready);
await page.waitForTimeout(700);
const timings = await page.evaluate(() => {
  const navigation = performance.getEntriesByType("navigation")[0];
  const resources = performance.getEntriesByType("resource");
  const paint = performance.getEntriesByType("paint").find(x => x.name === "first-contentful-paint");
  const totals = Object.fromEntries(["script", "css", "img", "video", "font"].map(type => [type, { requests: resources.filter(r => r.initiatorType === type).length, transferBytes: resources.filter(r => r.initiatorType === type).reduce((v, r) => v + r.transferSize, 0) }]));
  return { navigationMs: Math.round(navigation.duration), domInteractiveMs: Math.round(navigation.domInteractive), firstContentfulPaintMs: Math.round(paint?.startTime ?? 0), ...window.__launchVitals, totalTransferBytes: resources.reduce((v, r) => v + r.transferSize, 0), resources: resources.length, totals };
});
const metrics = (await cdp.send("Performance.getMetrics")).metrics;
const byName = Object.fromEntries(metrics.map(m => [m.name, m.value]));
const result = { conditions: "local static export, headless Chromium, CPU x4, warm network", timings, javascriptHeapMB: +(byName.JSHeapUsedSize / 1024 / 1024).toFixed(1), nodes: byName.Nodes, layoutCount: byName.LayoutCount, pageErrors: errors };
console.log(JSON.stringify(result, null, 2));
await writeFile("test-results/performance.json", JSON.stringify(result, null, 2));
await browser.close();
