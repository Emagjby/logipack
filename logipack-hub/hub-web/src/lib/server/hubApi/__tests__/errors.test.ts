import { describe, expect, test } from "bun:test";
import {
	HubApiError,
	hubApiErrorFromResponse,
	hubApiErrorFromThrowable,
	parseJsonOrThrowHubApiError,
} from "../errors";

describe("HubApiError model", () => {
	test("400 JSON body {code,message}", async () => {
		const response = new Response(
			JSON.stringify({ code: "BAD_INPUT", message: "nope" }),
			{
				status: 400,
				headers: { "content-type": "application/json" },
			},
		);

		const err = await hubApiErrorFromResponse({
			response,
			upstream: { method: "POST", path: "/x" },
		});

		expect(err).toBeInstanceOf(HubApiError);
		expect(err.status).toBe(400);
		expect(err.code).toBe("BAD_INPUT");
		expect(err.message).toBe("nope");
		expect(err.retryable).toBe(false);
	});

	test("403 non-JSON body", async () => {
		const response = new Response("forbidden", { status: 403 });

		const err = await hubApiErrorFromResponse({
			response,
			upstream: { method: "GET", path: "/secure" },
		});

		expect(err.status).toBe(403);
		expect(err.retryable).toBe(false);
		expect(err.code).toBe("UPSTREAM_ERROR_TEXT");
		expect(err.message).toBe("forbidden");
	});

	test("404 empty body", async () => {
		const response = new Response("", { status: 404 });

		const err = await hubApiErrorFromResponse({
			response,
			upstream: { method: "GET", path: "/missing" },
		});

		expect(err.status).toBe(404);
		expect(err.retryable).toBe(false);
		expect(err.code).toBe("UPSTREAM_ERROR");
		expect(err.message).toContain("404");
	});

	test("500 non-JSON", async () => {
		const response = new Response("kaboom", { status: 500 });

		const err = await hubApiErrorFromResponse({
			response,
			upstream: { method: "GET", path: "/boom" },
		});

		expect(err.status).toBe(500);
		expect(err.retryable).toBe(true);
		expect(err.code).toBe("UPSTREAM_ERROR_TEXT");
		expect(err.message).toBe("kaboom");
	});

	test("timeout/abort -> retryable", () => {
		const abortErr = new DOMException("Aborted", "AbortError");

		const err = hubApiErrorFromThrowable({
			err: abortErr,
			upstream: { method: "GET", path: "/slow" },
		});

		expect(err.status).toBe(0);
		expect(err.code).toBe("ABORTED");
		expect(err.retryable).toBe(true);
	});

	test("JSON parse failure on expected JSON responses", async () => {
		const response = new Response("not-json", {
			status: 200,
			headers: { "content-type": "application/json" },
		});

		const promise = parseJsonOrThrowHubApiError<{ ok: boolean }>({
			response,
			upstream: { method: "GET", path: "/ok" },
		});

		await expect(promise).rejects.toBeInstanceOf(HubApiError);
	});
});
