import { expect, test } from '@playwright/test';

test('home page has expected h1', async ({ page }) => {
	await page.goto('/');
	await expect(page.locator('.typst-doc')).toBeVisible();
	await expect(page.locator('h5:div')).toContainText("Hello Typst");
});
