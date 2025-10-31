import { createFetch } from 'openapi-hooks';

import type { paths } from './schema.gen';

export const baseUrl = new URL('/api/', import.meta.env.VITE_API_URL ?? window.location.origin);

// Extend paths to satisfy openapi-hooks Paths constraint
type ExtendedPaths = paths & {
    [key: string]: { [key: string]: any };
};

export const apiRequest = createFetch<ExtendedPaths>({
    baseUrl,
    async headers() {
        return {};
    },
    onError(error: { status: number }) {
        console.error('API Error:', error.status);
    },
}); 