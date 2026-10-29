import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const installer = "v0.16.1/NovyDeck-0.16.1-Setup.exe";
for (const width of [320, 390, 768, 1024, 1440]) {
  test(`responsive ${width}px: sections, images, no errors`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    const errors: string[] = [];
    page.on("pageerror", error => errors.push(error.message));
    page.on("response", response => { if (response.status() >= 400) errors.push(`${response.status()} ${response.url()}`); });
    await page.goto("/", { waitUntil: "networkidle" });
    await expect(page.getByRole("heading", { level: 1 })).toContainText("Chaque touche");
    await expect(page.locator("main section")).toHaveCount(6);
    await expect(page.locator(".story-row")).toHaveCount(5);
    await expect(page.locator(".bento-card")).toHaveCount(3);
    for (const card of await page.locator(".bento-card").all()) await card.scrollIntoViewIfNeeded();
    await expect.poll(() => page.locator(".bento-card img").evaluateAll(nodes => nodes.every(node => (node as HTMLImageElement).naturalWidth > 0))).toBe(true);
    const cards = await page.locator(".bento-card").evaluateAll(nodes => nodes.map(card => {
      const copy = card.querySelector(".bento-copy")!.getBoundingClientRect();
      const capture = card.querySelector("a")!.getBoundingClientRect();
      const image = card.querySelector("img")!;
      const box = card.getBoundingClientRect();
      return { gap: capture.top - copy.bottom, top: box.top, fit: getComputedStyle(image).objectFit, loaded: image.naturalWidth > 0 };
    }));
    expect(cards.every(card => card.gap >= 0 && card.gap <= 45 && card.fit === "contain" && card.loaded)).toBe(true);
    if (width >= 1024) expect(cards.every(card => Math.abs(card.top - cards[0].top) < 2)).toBe(true);
    await expect(page.locator(`a[href*='${installer}']`)).toHaveCount(2);
    for (const section of ["#top", "#story", "#features", "#compatibility", "#faq", "#download"]) {
      await page.locator(section).scrollIntoViewIfNeeded();
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${width}px overflow at ${section}`).toBe(true);
    }
    for (const image of ["v0161-onoff-cards-fr", "v0161-plugins-tab-fr"]) {
      const img = page.locator(`.story-visual img[src*='${image}']`);
      await img.scrollIntoViewIfNeeded();
      await expect.poll(() => img.evaluate(node => (node as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
      await expect(page.locator(`.story-visual a[href*='${image}']`)).toHaveCount(1);
    }
    if (width < 900) await expect(page.locator(".stage-deck canvas")).toHaveCount(0);
    else { await page.locator("#top").scrollIntoViewIfNeeded(); await expect(page.locator(".stage-deck canvas")).toBeVisible({ timeout: 15_000 }); }
    expect(errors).toEqual([]);
  });
}

test("language, keyboard navigation, FAQ and axe", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Aller au contenu" })).toBeFocused();
  await page.getByRole("button", { name: "Ouvrir le menu" }).click();
  await page.getByRole("navigation", { name: "Navigation principale" }).getByRole("link", { name: "Questions" }).click();
  await page.locator("#faq summary").first().click();
  await expect(page.locator("#faq details").first()).toHaveAttribute("open", "");
  await page.getByRole("button", { name: "Passer en anglais" }).click();
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Every key");
  await expect(page.locator(".story-row")).toHaveCount(5);
  await expect(page.locator("img[alt*='French off-screen Windows harness']")).toHaveCount(2);
  // Auditer après la fin des révélations au défilement, sans prendre leur état transitoire pour le contraste final.
  await page.waitForTimeout(1100);
  const results = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"]).analyze();
  expect(results.violations.filter(v => ["serious", "critical"].includes(v.impact ?? ""))).toEqual([]);
});

test("reduced motion keeps responsive WebGL without autonomous motion", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/", { waitUntil: "networkidle" });
  await expect(page.locator(".stage-deck canvas")).toBeVisible();
  expect(await page.evaluate(() => getComputedStyle(document.documentElement).scrollBehavior)).toBe("auto");
});

test("WebGL fits the stage and blue controls have no warm accent", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/", { waitUntil: "networkidle" });
  const canvas = page.locator(".stage-deck canvas");
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  expect(box?.width).toBeGreaterThan(200);
  expect(box?.width).toBeLessThan(1000);
  const button = await page.locator(".button-primary").first().evaluate(el => getComputedStyle(el).backgroundColor);
  expect(button).toBe("rgb(0, 100, 189)");
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(1440);
});
