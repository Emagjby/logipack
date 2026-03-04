// Generic DTO type for endpoints we haven't modelled yet.
export type UnknownDto = Record<string, unknown>;

export type UserDto = {
	id: string;
	email: string;
	name?: string | null;
};

export type UserDetail = {
	email: string;
	name?: string | null;
};
