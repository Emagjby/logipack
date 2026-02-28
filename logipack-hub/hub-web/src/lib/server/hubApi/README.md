# hubApi Usage Guide

This folder is the only place where `hub-web` should talk to `hub-api`.

## Required pattern

1. In `+page.server.ts` or `+layout.server.ts`, create a client from SSR context.
2. Call an endpoint with the client.
3. Map raw DTO data to app-safe data with mapper functions.
4. Handle `HubApiError` in one place in the route load/action.

Example:

```ts
import {
  createHubApiClient,
  mapOfficeListItemDtoToOfficeListItem,
} from "$lib/server/hubApi";

export async function load({ fetch, locals }) {
  const client = createHubApiClient({
    fetch,
    locals,
    baseUrl: process.env.HUB_API_BASE ?? "",
  });

  try {
    const res = await client.get<{ offices: unknown[] }>("/admin/offices");
    const offices = res.data.offices.map((dto) =>
      mapOfficeListItemDtoToOfficeListItem(dto),
    );
    return { offices };
  } catch (err) {
    // Route-level mapping of HubApiError to UI state
    return { offices: [], loadError: true };
  }
}
```

## Forbidden pattern

- Do not call `fetch(${HUB_API_BASE}...)` directly in routes.
- Do not forward `Cookie` headers to `hub-api`.
- Do not cast raw JSON in routes with ad hoc `as SomeType`.

## Import rule

Import from the barrel file only:

```ts
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
```

Do not deep import from `dto/*`, `mappers/*`, `errors.ts`, or `httpClient.ts`.

## Reviewer checklist

- Route uses `createHubApiClient` from this module.
- Route does not call direct `fetch` to `hub-api`.
- Route maps DTOs through mapper functions.
- Route handles `HubApiError` explicitly.
