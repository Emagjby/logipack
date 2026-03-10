import { describe, expect, test } from "bun:test";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";

const APP_ROOT = path.resolve(process.cwd(), "src/routes/[lang]/app");
const EN_LOCALE_PATH = path.resolve(process.cwd(), "src/lib/i18n/locales/en.json");
const BG_LOCALE_PATH = path.resolve(process.cwd(), "src/lib/i18n/locales/bg.json");
const TRANSLATION_KEY_PATTERN =
	/^(admin|employee|shipment|shipments|office|client|reports|error)\.[a-z0-9_.-]+$/;

function walk(dir: string): string[] {
	const entries = readdirSync(dir, { withFileTypes: true });
	return entries.flatMap((entry) => {
		const fullPath = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			return walk(fullPath);
		}
		return [fullPath];
	});
}

function hasLocaleKey(locale: Record<string, unknown>, key: string): boolean {
	return Object.prototype.hasOwnProperty.call(locale, key);
}

describe("app cutover regression checks", () => {
	test("page server routes contain no mock imports or mock helper usage", () => {
		const pageServerFiles = walk(APP_ROOT).filter((file) =>
			file.endsWith("+page.server.ts"),
		);

		for (const file of pageServerFiles) {
			const source = readFileSync(file, "utf8");
			expect(source).not.toMatch(/\$lib\/server\/mock/);
			expect(source).not.toMatch(/\b(?:listMock|getMock|createMock|updateMock|deleteMock|assignMock)\b/);
		}
	});

	test("route error/message assignments use explicit i18n keys", () => {
		const routeServerFiles = walk(APP_ROOT).filter(
			(file) => file.endsWith("+page.server.ts") || file.endsWith("+layout.server.ts"),
		);
		const assignmentPattern =
			/\b(?:message|submitError|changeStatusError)\s*:\s*"([^"]+)"/g;

		for (const file of routeServerFiles) {
			const source = readFileSync(file, "utf8");

			for (const match of source.matchAll(assignmentPattern)) {
				const key = match[1] ?? "";
				expect(key).toBe(key.trim());
				expect(key).toMatch(TRANSLATION_KEY_PATTERN);
			}
		}
	});

	test("all route-referenced i18n keys exist in both EN and BG locales", () => {
		const routeServerFiles = walk(APP_ROOT).filter(
			(file) => file.endsWith("+page.server.ts") || file.endsWith("+layout.server.ts"),
		);
		const en = JSON.parse(readFileSync(EN_LOCALE_PATH, "utf8")) as Record<
			string,
			unknown
		>;
		const bg = JSON.parse(readFileSync(BG_LOCALE_PATH, "utf8")) as Record<
			string,
			unknown
		>;
		const keyPattern =
			/"((?:admin|employee|shipment|shipments|office|client|reports|error)\.[^"]+)"/g;
		const keys = new Set<string>();

		for (const file of routeServerFiles) {
			const source = readFileSync(file, "utf8");

			for (const match of source.matchAll(keyPattern)) {
				const key = match[1] ?? "";
				if (TRANSLATION_KEY_PATTERN.test(key)) {
					keys.add(key);
				}
			}
		}

		for (const key of keys) {
			expect(hasLocaleKey(en, key)).toBeTrue();
			expect(hasLocaleKey(bg, key)).toBeTrue();
		}
	});
});
