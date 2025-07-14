# OpenAPI TypeScript + React Query Setup

This setup provides end-to-end type safety from your OpenAPI backend to your React frontend using generated TypeScript types and React Query hooks.

## 1. Dependencies

```json
{
  "dependencies": {
    "openapi-hooks": "0.0.6",
    "@tanstack/react-query": "^5.64.1"
  },
  "devDependencies": {
    "openapi-typescript": "^7.6.1"
  }
}
```

## 2. Schema Generation Script

Add this script to your `package.json`:

```json
{
  "scripts": {
    "api-schema": "openapi-typescript http://localhost:3000/openapi.json --output ./src/api/schema.gen.ts",
    "dev": "pnpm api-schema && vite --host"
  }
}
```

This automatically generates TypeScript types from your OpenAPI spec whenever you run the dev server.

## 3. API Client Setup

Create `src/api/api.ts`:

```typescript
import { createFetch } from 'openapi-hooks';
import { toast } from 'sonner';

import { paths } from './schema.gen';

export const baseUrl = new URL('/api/', import.meta.env.VITE_API_URL ?? window.location.origin);

export const useApi = createFetch<paths>({
    baseUrl,
    async headers() {
        const token = localStorage.getItem('auth_token');

        return {
            ...(token && { Authorization: `Bearer ${token}` }),
        };
    },
    onError(error: { status: number }) {
        // If we get a 401, clear auth data
        if (error.status === 401) {
            localStorage.removeItem('auth_token');
            localStorage.removeItem('auth_user');
            localStorage.removeItem('auth_expires_at');

            // Let the auth system handle the state update
            window.dispatchEvent(new Event('auth-cleared'));
        }

        if (error.status === 429) {
            toast.error('Request throttled, please wait a moment before retrying');
        }
    },
});
```

## 4. Usage in React Components

The `useApi` hook provides fully typed React Query hooks for all your API endpoints:

```typescript
import { useApi } from '../api/api';

const MyComponent = () => {
    // GET requests become useQuery hooks
    const { data: topics, isLoading } = useApi('/api/topics', {
        query: { limit: 10 }
    });

    // POST/PUT/DELETE requests become useMutation hooks
    const createTopicMutation = useApi('/api/topics', {
        method: 'POST'
    });

    const handleCreateTopic = async (topicData) => {
        await createTopicMutation.mutateAsync({
            body: topicData
        });
    };

    return (
        <div>
            {isLoading ? 'Loading...' : topics?.map(topic => (
                <div key={topic.id}>{topic.title}</div>
            ))}
        </div>
    );
};
```

## 5. How It Works

1. **openapi-typescript**: Generates TypeScript types from your OpenAPI specification
   - Creates a `paths` interface that maps all your API endpoints
   - Includes request/response types, query parameters, path parameters, etc.

2. **openapi-hooks**: Provides React Query integration
   - `createFetch<paths>()` creates a typed API client
   - Automatically generates `useQuery` hooks for GET requests
   - Automatically generates `useMutation` hooks for POST/PUT/DELETE requests
   - Provides full TypeScript autocomplete and type checking

3. **Type Safety**:
   - Endpoint URLs are type-checked
   - Request bodies are validated
   - Response data is properly typed
   - Query parameters and path parameters are type-safe

## 6. Additional Features

- **Automatic Authentication**: Headers function runs on every request
- **Global Error Handling**: `onError` callback handles common error scenarios
- **Environment Configuration**: Supports different API URLs for dev/prod
- **React Query Integration**: Automatic caching, background refetching, optimistic updates

This setup eliminates the need to manually write API client code or type definitions, as everything is generated from your OpenAPI spec and stays in sync automatically.

## How to write a hook

```typescript
const getUser = (discourseId: string, username: string) => {
    return {
        queryKey: ['user', discourseId, username],
        queryFn: async () => {
            const response = await useApi('/du/{discourse_id}/{username}', 'get', {
                path: {
                    discourse_id: discourseId,
                    username: username,
                },
            });

            return response.data;
        },
    };
};


export const useUser = (discourseId: string, username: string) =>
    useQuery(getUser(discourseId, username));
```

please ensure to write the above in a file such as `/api/user.ts`.
