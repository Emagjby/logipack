import { describe, expect, test } from "bun:test";
import { parseSession } from "$lib/server/session.server";

function shouldAttemptRefresh(args: {
	refresh_token?: string;
	expires_at: number;
	nowSec: number;
	skewSec: number;
}): boolean {
	const rt = args.refresh_token;
	if (!rt || rt.trim() === "") return false;
	return args.expires_at > 0 && args.expires_at - args.nowSec < args.skewSec;
}

describe("parseSession", () => {
	test("valid payload with refresh token parses correctly", () => {
		const payload = {
			access_token: "tok",
			refresh_token: "ref",
			expires_at: 3600,
			role: "admin",
			name: "andi",
			email: "g@example.com",
		};

		const s = parseSession(payload);
		expect(s).not.toBeNull();
		expect(s?.refresh_token).toBe("ref");
	});

	test("valid payload without refresh token passes correctly", () => {
		const payload = {
			access_token: "tok",
			expires_at: 3600,
			role: "admin",
			name: "andi",
			email: "g@example.com",
		};

		const s = parseSession(payload);
		expect(s).not.toBeNull();
		expect(s?.refresh_token).toBeUndefined();
	});

	test("invalid payload returns null", () => {
		expect(parseSession({})).toBeNull();
		expect(parseSession(null)).toBeNull();
		expect(
			parseSession({
				access_token: "",
				expires_at: 3600,
				role: "admin",
				name: "andi",
				email: "g@example.com",
			}),
		).toBeNull();
	});

	test("no refresh attempt when refresh token is missing or empty", () => {
		expect(
			shouldAttemptRefresh({
				refresh_token: undefined,
				expires_at: 100,
				nowSec: 90,
				skewSec: 30,
			}),
		).toBe(false);

		expect(
			shouldAttemptRefresh({
				refresh_token: "   ",
				expires_at: 100,
				nowSec: 90,
				skewSec: 30,
			}),
		).toBe(false);
	});

	test("refresh attempt only when near expiry and token present", () => {
		expect(
			shouldAttemptRefresh({
				refresh_token: "ref",
				expires_at: 100,
				nowSec: 90,
				skewSec: 30,
			}),
		).toBe(true);

		expect(
			shouldAttemptRefresh({
				refresh_token: "ref",
				expires_at: 1000,
				nowSec: 0,
				skewSec: 30,
			}),
		).toBe(false);
	});
});
