---
id: methodology
title: Frontend Lead — Development Methodology
domain: experts
category: frontend-lead
difficulty: intermediate
tags: [architecture, client, component, error, experts, handling, management, methodology]
quality_score: 70
last_updated: 2026-06-15
---
# Frontend Lead — Development Methodology

## Component Architecture

### Component Categories
1. **Primitives** (atoms): Button, Input, Badge, Avatar, Icon
   - No business logic, only presentation
   - Accept variants via props (size, color, disabled)
   - Fully accessible (keyboard, ARIA)

2. **Composites** (molecules): SearchBar, FormField, Card, Modal
   - Combine 2-3 primitives
   - May have internal state (open/closed, input value)
   - Still reusable across features

3. **Features** (organisms): LoginForm, DashboardHeader, UserList
   - Business logic lives here
   - Connect to API / state management
   - Specific to one feature

4. **Pages** (templates): /dashboard, /settings, /auth/login
   - Compose features into a full layout
   - Handle routing, auth guards, data fetching

### Component File Structure
```
components/
  Button/
    Button.tsx        # component
    Button.test.tsx   # unit tests
    Button.stories.tsx # storybook (if used)
    index.ts          # re-export
```

### Props Design Rules
- Use interface, not inline types
- Required props first, optional last
- Sensible defaults for optional props
- Event handlers: `onX` naming (onClick, onChange, onSubmit)
- Children for composition, not deep prop drilling
- No more than 7 props (split into smaller components if needed)

## State Management

### Where State Lives
| State type | Storage | Example |
|---|---|---|
| UI state (local) | useState / ref | modal open, input value, accordion expanded |
| Form state | form library | field values, validation, dirty/touched |
| Server state | query cache (React Query / SWR) | API data, loading/error states |
| Global app state | context / store | auth user, theme, locale |
| URL state | search params / path | current page, filters, sort order |

### Rules
- Default to local state. Only lift when two+ components need it.
- Server state is NOT client state — use a cache library, not Redux/Zustand for API data.
- URL state for anything the user might bookmark or share.
- Never store derived values — compute them on render.

## API Client Pattern

### Centralized fetch wrapper
```typescript
// lib/api.ts
const API_BASE = process.env.NEXT_PUBLIC_API_URL;

export async function apiFetch<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: {
      'Content-Type': 'application/json',
      ...getAuthHeader(),
      ...options?.headers,
    },
    ...options,
  });
  if (!res.ok) {
    const error = await res.json().catch(() => ({ message: res.statusText }));
    throw new ApiError(res.status, error.message, error.details);
  }
  return res.json();
}
```

### Per-resource API functions
```typescript
// api/users.ts
export const usersApi = {
  list: (params?: ListParams) => apiFetch<User[]>('/users', { params }),
  get: (id: string) => apiFetch<User>(`/users/${id}`),
  create: (data: CreateUser) => apiFetch<User>('/users', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: Partial<User>) => apiFetch<User>(`/users/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
  delete: (id: string) => apiFetch<void>(`/users/${id}`, { method: 'DELETE' }),
};
```

## Error Handling

### Error Boundary (global)
Catches rendering errors, shows fallback UI, reports to error tracking.

### API Error Handling (per-request)
```typescript
try {
  const data = await usersApi.create(formData);
  // success: redirect or show toast
} catch (error) {
  if (error instanceof ApiError) {
    if (error.status === 422) {
      // validation: show field-level errors
      setFieldErrors(error.details);
    } else if (error.status === 409) {
      // conflict: "email already exists"
      showToast('error', error.message);
    } else {
      // other API error
      showToast('error', 'Something went wrong');
    }
  } else {
    // network error
    showToast('error', 'Unable to connect to server');
  }
}
```

### Loading States
- Skeleton screens for initial load (not spinners)
- Inline loading for mutations (button shows spinner, text changes to "Saving...")
- Optimistic updates for fast-feeling UI (update UI first, then sync with server)

### Empty States
Every list/table/grid must have:
- First-time empty: "No items yet. Create your first X."
- Filtered empty: "No results match your filters."
- Error empty: "Failed to load. [Retry button]"

## Performance Checklist

- [ ] Images: lazy loaded, responsive sizes, modern format (WebP/AVIF)
- [ ] Fonts: preloaded, `font-display: swap`, subset if possible
- [ ] JavaScript: code-split by route, tree-shaken, no unused dependencies
- [ ] CSS: purged unused styles, critical CSS inlined
- [ ] API calls: deduplicated (cache library), prefetched on hover
- [ ] Lists: virtualized if > 100 items (react-virtual / tanstack-virtual)
- [ ] Bundle size: < 200KB gzipped for initial load
- [ ] Core Web Vitals: LCP < 2.5s, FID < 100ms, CLS < 0.1
