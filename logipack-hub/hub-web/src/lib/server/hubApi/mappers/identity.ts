import type { MeResponseDto } from "../dto/identity";

export type LpRole = "admin" | "employee" | "";

export function mapMeRole(dto: MeResponseDto): LpRole {
	const role = dto.role.trim();
	if (role === "admin" || role === "employee") {
		return role;
	}
	return "";
}
