import { test, expect, Page } from "@playwright/test";

async function* joinGame(page: Page, code: string, name: string) {
  await page.goto("https://localhost:8788");

  await page.getByRole("link", { name: "Play" }).click();
  await page.getByRole("textbox").click();
  await page.getByRole("textbox").fill(code);
  await page.getByRole("button", { name: "Join" }).click();

  yield;

  await page.getByRole("textbox").click();
  await page.getByRole("textbox").fill(name);
  await page.getByRole("button", { name: "Change" }).click();

  yield;

  await page.getByRole("button", { name: "Ready" }).click();

  yield;

  await page.getByRole("button", { name: "+" }).first().click();
  await page.getByRole("button", { name: "+" }).first().click();
  await page.getByRole("button", { name: "+" }).nth(1).click();
  await page.getByRole("button", { name: "Place Bets" }).click();
  await page.getByText("700").first();
  await page.getByText("300").click();
  await page.getByText("700").nth(1).click();
  await page.getByText("Bet 200").click();
  await page.getByText("Bet 100").click();
  await page.getByText("Bet 0").click();
}

test("can setup and join lobby", async ({ browser }) => {
  const hostContext = await browser.newContext();
  const hostPage = await hostContext.newPage();

  const aliceContext = await browser.newContext();
  const alicePage = await aliceContext.newPage();

  const bobContext = await browser.newContext();
  const bobPage = await bobContext.newPage();

  await hostPage.goto("https://localhost:8788");
  await hostPage.getByRole("button", { name: "Host" }).click();

  const code_locator = hostPage.getByTestId("game_code");
  await expect(code_locator).toHaveText(/[A-Z0-9]{6}/);

  const code = await code_locator.innerText();

  const players = [
    joinGame(alicePage, code, "Alice"),
    joinGame(bobPage, code, "Bob"),
  ];

  loop: while (true) {
    for (const player of players) {
      if ((await player.next()).done) {
        break loop;
      }
    }
  }
});
