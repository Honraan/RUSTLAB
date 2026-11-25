import assert from "node:assert/strict";
import { setTimeout as wait } from "node:timers/promises";
import { chromium } from "playwright-core";

let browser;
let page;
try {
  const endpoint = "http://127.0.0.1:9333";
  for (let attempt = 0; attempt < 20 && !page; attempt++) {
    try {
      browser ??= await chromium.connectOverCDP(endpoint, { timeout: 1500 });
      page = browser.contexts().flatMap((context) => context.pages()).find((candidate) => /^https?:\/\/tauri\.localhost(?:\/|$)/.test(candidate.url()));
    } catch {
      // WebView2 may still be starting.
    }
    if (!page) await wait(700);
  }
  assert(page, "A WebView2 instalada não expôs a página local.");
  await page.getByRole("heading", { name: /Aprenda Rust.*escrevendo Rust/ }).waitFor({ timeout: 15000 });
  if (process.argv.includes("--restored")) {
    await page.getByText("20 XP", { exact: true }).waitFor();
    await page.getByRole("button", { name: "Retomar trilha" }).waitFor();
  } else {
    await page.getByRole("button", { name: "Começar a aprender" }).click();
    await page.getByRole("heading", { name: "O que é programar?" }).waitFor();
    await page.getByText(/Programar é escrever/).first().waitFor();
    await page.getByRole("radio", { name: "Compreender instruções, sequência e exemplos reais de software." }).check();
    await page.getByRole("button", { name: "Conferir respostas" }).click();
    await page.getByText("Lição concluída. +20 XP.").waitFor();
    await page.getByText("20 XP", { exact: true }).waitFor();
    await page.keyboard.press("F9");
    await page.getByRole("heading", { name: "Bibliotecas", exact: true }).waitFor();
    await page.getByRole("heading", { name: "bevy 0.20" }).waitFor();
    const serious = await page.locator(".notice").allTextContents();
    assert.deepEqual(serious, []);
    await page.screenshot({ path: "windows-artifact/rustlab-webview2.png" });
  }
} finally {
  await browser?.close();
}
