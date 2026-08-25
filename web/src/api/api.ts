import { createFetch } from "openapi-hooks";

import { authHeaders } from "./auth";
import type { paths } from "./schema.gen";

export const baseUrl = new URL("/api/", import.meta.env.VITE_API_URL ?? globalThis.location.origin);

// A mapped type over the generated interface satisfies openapi-hooks' index-signature
// constraint while keeping every response type intact.
type ExtendedPaths = { [Route in keyof paths]: paths[Route] };

export const apiRequest = createFetch<ExtendedPaths>({
  baseUrl,
  async headers() {
    return authHeaders();
  },
  onError(error: { status: number; }) {
    console.error("API error:", error.status);
  },
});
