# DevOps — Methodology

## CI/CD Pipeline Design

### Pipeline Stages
```
commit → lint → test → build → deploy:staging → smoke-test → deploy:production → monitor
```

### Stage Details

**Lint** (< 1 min)
- TypeScript: `tsc --noEmit`
- ESLint: `eslint --max-warnings 0`
- Prettier: `prettier --check`
- Fail fast: any lint failure blocks the pipeline

**Test** (< 5 min)
- Unit tests: `jest` / `vitest` / `cargo test`
- Coverage gate: fail if below threshold (80% for business logic)
- Parallel execution when possible

**Build** (< 3 min)
- Production build with minification
- Generate source maps (for error tracking, not served to users)
- Output bundle size check (fail if > budget)

**Deploy: Staging**
- Automatic on every merged PR to main
- Same infrastructure as production (just scaled down)
- Seeded with realistic test data

**Smoke Test**
- Automated: hit key endpoints, verify 200 responses
- Verify database migration ran successfully
- Check external service connections (payment, email, etc)

**Deploy: Production**
- Manual approval gate (or auto after staging smoke passes)
- Rolling deploy (no downtime)
- Database migrations run before new code deploys

**Monitor**
- Watch error rate for 15 minutes after deploy
- Auto-rollback if error rate > 1%

## Environment Strategy

| Environment | Purpose | Data | Access |
|---|---|---|---|
| local | Development | Seed data | Developer |
| staging | Pre-production testing | Anonymized prod copy | Team |
| production | Live users | Real data | Restricted |

### Environment Variables
- Same variable names across all environments
- Different VALUES per environment (never different variable names)
- `.env.example` committed with placeholder values
- Actual `.env` files NEVER committed

## Docker Standards

### Dockerfile Best Practices
```dockerfile
# Multi-stage build
FROM node:20-slim AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --production=false
COPY . .
RUN npm run build

FROM node:20-slim
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
EXPOSE 3000
CMD ["node", "dist/server.js"]
```

Rules:
- Multi-stage builds (separate build/runtime)
- Pin base image versions (`node:20-slim`, not `node:latest`)
- `npm ci` not `npm install` (deterministic)
- Non-root user in production image
- `.dockerignore` excludes node_modules, .git, .env

## Monitoring & Alerting

### Four Golden Signals
| Signal | What to measure | Alert threshold |
|---|---|---|
| **Latency** | p95 response time | > 500ms for 5 min |
| **Traffic** | Requests per second | Drop > 50% from baseline |
| **Errors** | 5xx error rate | > 1% for 5 min |
| **Saturation** | CPU/memory usage | > 80% for 10 min |

### Health Check Endpoint
```
GET /api/health
→ 200 { "status": "healthy", "version": "1.2.3", "uptime": 86400 }
→ 503 { "status": "unhealthy", "checks": { "database": "timeout" } }
```

Check: database connection, cache connection, external API reachability

### Logging in Production
- Structured JSON logs (not plain text)
- Include: timestamp, level, requestId, userId, message
- Never log: passwords, tokens, PII, credit card numbers
- Log to stdout (let the platform handle aggregation)

## Rollback Strategy

### Criteria for rollback
- Error rate > 1% sustained for 5 minutes
- Any 500 on critical path (checkout, login)
- Performance regression > 2x baseline latency

### Rollback steps
1. Route traffic to previous version (< 1 min)
2. Verify previous version is healthy
3. Investigate root cause on the failed version
4. Fix → test on staging → re-deploy

### Database rollback
- Every migration has a DOWN migration
- Test DOWN migration BEFORE deploying UP
- Never drop columns immediately — deprecate, deploy, then drop in next release

## Security in Deployment

- [ ] All secrets in environment variables / secret manager (never in code)
- [ ] HTTPS everywhere (HSTS with 1-year max-age)
- [ ] Database not publicly accessible (VPC/private network only)
- [ ] SSH access via bastion/jump server only
- [ ] Automated dependency vulnerability scanning
- [ ] Container images scanned for CVEs before deploy
