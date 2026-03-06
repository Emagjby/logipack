import type { MeResponseDto } from "../dto/identity";

export type LpRole = "admin" | "employee" | "";

export type MeContext = {
	role: LpRole;
	office_ids: string[];
	current_office_id: string | null;
};

export function mapMeRole(dto: MeResponseDto): LpRole {
	const role = dto.role.trim();
	if (role === "admin" || role === "employee") {
		return role;
	}
	return "";
}

export function mapMeContext(dto: MeResponseDto): MeContext {
	const role = mapMeRole(dto);
	const office_ids = Array.isArray(dto.office_ids)
		? dto.office_ids
				.filter((id): id is string => typeof id === "string")
				.map((id) => id.trim())
				.filter(Boolean)
		: [];

	const current_office_id =
		typeof dto.current_office_id === "string" && dto.current_office_id.trim()
			? dto.current_office_id.trim()
			: null;

	return {
		role,
		office_ids,
		current_office_id,
	};
}
