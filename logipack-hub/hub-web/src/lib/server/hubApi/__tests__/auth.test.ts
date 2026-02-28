import { describe, test, expect } from "bun:test";
import { getAccessTokenFromLocals } from "../auth";

describe("getAccessTokenFromLocals", () => {
	test("token returned if exists", () => {
		const locals = {
			session: {
				access_token: "valid_token",
				expires_at: 123,
				refresh_token: "refresh_token",
				role: "",
				name: "",
				email: "",
			},
		} as any;

		expect(getAccessTokenFromLocals(locals)).toBe("valid_token");
	});

	test("missing session returns null", () => {
		const locals = { session: null } as any;
		expect(getAccessTokenFromLocals(locals)).toBeNull();
	});

	test("malformed token returns null", () => {
		const locals = {
			session: {
				access_token: 123,
			},
		} as any;

		expect(getAccessTokenFromLocals(locals)).toBeNull();
	});

	test("empty or whitespace token returns null", () => {
		const locals = {
			session: {
				access_token: "   ",
			},
		} as any;

		expect(getAccessTokenFromLocals(locals)).toBeNull();
	});
});
