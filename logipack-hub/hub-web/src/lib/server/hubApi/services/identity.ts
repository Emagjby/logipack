import type { EnsureUserRequestDto, MeResponseDto } from "../dto/identity";
import {
	mapMeContext,
	mapMeRole,
	type LpRole,
	type MeContext,
} from "../mappers/identity";
import { HubApiError } from "../errors";
import type { HubApiClient } from "../index";

/**
 * POST /ensure-user
 * hub-api requires email and phone
 * throws HubApiError on failure
 */
export async function ensureUser(
	client: HubApiClient,
	dto: EnsureUserRequestDto,
	timeoutMs = 10_000,
): Promise<void> {
	await client.post<unknown>("/ensure-user", dto, { timeoutMs });
}

/**
 * GET /me
 * throws HubApiError on failure
 */
export async function getMe(
	client: HubApiClient,
	timeoutMs = 5_000,
): Promise<LpRole> {
	try {
		const res = await client.get<MeResponseDto>("/me", { timeoutMs });
		return mapMeRole(res.data);
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return "";
		}
		throw e;
	}
}

export async function getMeContext(
	client: HubApiClient,
	timeoutMs = 5_000,
): Promise<MeContext> {
	try {
		const res = await client.get<MeResponseDto>("/me", { timeoutMs });
		return mapMeContext(res.data);
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return { role: "", office_ids: [], current_office_id: null };
		}
		throw e;
	}
}
