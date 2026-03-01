import type { EnsureUserRequestDto, MeResponseDto } from "../dto/identity";
import { mapMeRole, type LpRole } from "../mappers/identity";
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
