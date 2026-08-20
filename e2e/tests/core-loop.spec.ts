import { expect, test, type Page } from "@playwright/test";

/**
 * Waits for the WebAssembly bundle to boot.
 *
 * `networkidle` is the wrong signal for a wasm application: the network goes
 * quiet while the module is still instantiating, so a test can look at an empty
 * page and call it a failure. Waiting for something the app itself rendered is
 * the only reliable answer.
 */
async function ready(page: Page) {
    await expect(page.locator(".header__name")).toBeVisible();
}

async function addIdea(page: Page, name: string, description: string, tag?: string) {
    await page.goto("/collections");
    await ready(page);

    await page.getByRole("button", { name: "New Idea" }).first().click();

    await page.getByPlaceholder("Give your idea a name").fill(name);
    await page.getByPlaceholder("Describe the idea").fill(description);

    if (tag) {
        await page.getByRole("button", { name: "+ New Tag" }).click();
        await page.getByPlaceholder("Tag").fill(tag);
        await page.getByPlaceholder("Tag").press("Enter");
        await expect(page.locator(".tag", { hasText: tag })).toBeVisible();
    }

    await page.getByRole("button", { name: "Save" }).click();
    await expect(page.locator(".idea__title")).toContainText(name);
}

test("the home page offers a random idea and the core loop acts on it", async ({ page }) => {
    await addIdea(page, "Learn to sail", "Find a club on the coast.", "outdoors");

    await page.goto("/");
    await ready(page);

    await expect(page.locator(".idea__title")).toBeVisible();

    await page.getByRole("button", { name: "Mark as Done" }).click();
    await expect(page.getByRole("button", { name: "Undo" })).toBeVisible();

    await page.getByRole("button", { name: "Undo" }).click();
    await expect(page.getByRole("button", { name: "Mark as Done" })).toBeVisible();
});

test("an idea's markdown is rendered, and raw HTML in it is not", async ({ page }) => {
    await addIdea(
        page,
        "Read more",
        "A description with **bold** text.\n\n<img src=x onerror=alert(1)>",
    );

    await expect(page.locator(".idea__description strong")).toHaveText("bold");
    await expect(page.locator(".idea__description img")).toHaveCount(0);
});

test("the manage view lists ideas and marks them complete", async ({ page }) => {
    await addIdea(page, "Bake sourdough", "Start a starter.", "cooking");

    await page.goto("/collections");
    await ready(page);
    await page.getByRole("button", { name: "Manage" }).first().click();

    const row = page.locator("tr", { hasText: "Bake sourdough" });
    await expect(row).toBeVisible();

    await row.getByRole("button", { name: "Mark as done" }).click();
    await expect(row).toHaveClass(/success-row/);
});

test("a new collection can be created and reached from the list", async ({ page }) => {
    await page.goto("/collections/new");
    await ready(page);

    await page.getByPlaceholder("Give your collection a name").fill("Weekend Plans");
    await page.getByRole("button", { name: "Save" }).click();

    // Creating a collection lands on its manage view.
    await expect(page.locator(".manage__name")).toHaveText("Weekend Plans");

    await page.goto("/collections");
    await expect(page.getByText("Weekend Plans")).toBeVisible();
});

test("inviting somebody who has never signed in says so", async ({ page }) => {
    await page.goto("/collection/00000000000000000000000000000000/invite");
    await ready(page);

    await page.getByPlaceholder("someone@example.com").fill("nobody@example.com");

    await expect(page.locator(".form-field__hint")).toContainText("has used Rex yet");
    await expect(page.getByRole("button", { name: "Invite" })).toBeDisabled();
});

test("deleting an idea removes it", async ({ page }) => {
    await addIdea(page, "Doomed idea", "This one does not survive.");

    await page.getByRole("button", { name: "Delete" }).click();

    await page.goto("/collections");
    await ready(page);
    await page.getByRole("button", { name: "Manage" }).first().click();

    await expect(page.locator("tr", { hasText: "Doomed idea" })).toHaveCount(0);
});
