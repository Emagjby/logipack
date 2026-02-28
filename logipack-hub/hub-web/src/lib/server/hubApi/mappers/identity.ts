import type { MeDto } from "../dto/identity";

export type Me = Record<string, never>;

export function mapMeDtoToMe(_dto: MeDto): Me {
	return {};
}
