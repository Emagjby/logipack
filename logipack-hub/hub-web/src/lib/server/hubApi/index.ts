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
export * from "./dto/audit";
export * from "./dto/reports";
export * from "./dto/analytics";

export * from "./mappers/offices";
export * from "./mappers/clients";
export * from "./mappers/employees";
export * from "./mappers/shipments";
export * from "./mappers/identity";
export * from "./mappers/audit";
export * from "./mappers/reports";
export * from "./mappers/analytics";

export * from "./services/identity";
export * from "./services/offices";
export * from "./services/clients";
export * from "./services/employees";
export * from "./services/shipments";
export * from "./services/audit";
export * from "./services/reports";
export * from "./services/analytics";
