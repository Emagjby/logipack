import type { LpSession } from "$lib/server/session.server";

declare global {
	namespace App {
		interface Locals {
			lang?: "en" | "bg";
			session: LpSession | null;
		}
	}
}

export { };
