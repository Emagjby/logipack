import type { createHubApiClient } from "./httpClient";

export type {
  HttpMethod,
  HubApiRequestOptions,
  HubApiSuccess,
  HubApiFailure,
} from "./types";

export { getAccessTokenFromLocals } from "./auth";
export {
  HubApiError,
  hubApiErrorFromResponse,
  hubApiErrorFromThrowable,
  parseJsonOrThrowHubApiError,
} from "./errors";

export { createHubApiClient } from "./httpClient";
export type HubApiClient = ReturnType<typeof createHubApiClient>;

export * from "./normalizers";

export * from "./dto/common";
export * from "./dto/offices";
export * from "./dto/clients";
export * from "./dto/employees";
export * from "./dto/shipments";
export * from "./dto/identity";

export * from "./mappers/offices";
export * from "./mappers/clients";
export * from "./mappers/employees";
export * from "./mappers/shipments";
export * from "./mappers/identity";

export * from "./services/identity";
export * from "./services/offices";
export * from "./services/clients";
export * from "./services/employees";
