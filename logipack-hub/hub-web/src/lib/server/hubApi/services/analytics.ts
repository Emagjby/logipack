import type { HubApiClient } from "../index";
import type {
	AdminOverviewResponseDto,
	EmployeeOverviewResponseDto,
} from "../dto/analytics";
import type { AdminOverview, EmployeeOverview } from "../mappers/analytics";
import {
	mapAdminOverviewResponseDto,
	mapEmployeeOverviewResponseDto,
} from "../mappers/analytics";

export type AnalyticsSpan = "7d" | "30d" | "90d";

export async function getAdminOverview(
	client: HubApiClient,
	span: AnalyticsSpan = "30d",
	timeoutMs = 10_000,
): Promise<AdminOverview> {
	const res = await client.get<AdminOverviewResponseDto>(
		`/analytics/admin/overview?span=${span}`,
		{ timeoutMs },
	);
	return mapAdminOverviewResponseDto(res.data);
}

export async function getEmployeeOverview(
	client: HubApiClient,
	span: AnalyticsSpan = "30d",
	timeoutMs = 10_000,
): Promise<EmployeeOverview> {
	const res = await client.get<EmployeeOverviewResponseDto>(
		`/analytics/employee/overview?span=${span}`,
		{ timeoutMs },
	);
	return mapEmployeeOverviewResponseDto(res.data);
}
