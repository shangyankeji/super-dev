# Super Dev Spec-Driven Development

This project uses Super Dev's Spec-Driven Development (SDD) workflow.

## Workflow

1. **Propose**: Create a change proposal with `super-dev spec propose <id>`
2. **Review**: Review the generated specs and tasks
3. **Implement**: Work through the tasks with AI assistance
4. **Archive**: Archive the change with `super-dev spec archive <id>`

## Directory Structure

```
.super-dev/
├── specs/          # Current specifications (source of truth)
├── changes/        # Proposed changes (proposals + tasks + deltas)
└── archive/        # Archived changes
```

## Commands

- `super-dev spec list` - List all changes
- `super-dev spec show <id>` - Show change details
- `super-dev spec propose <id>` - Create new change proposal
- `super-dev spec apply <id>` - Start implementing a change
- `super-dev spec archive <id>` - Archive completed change

## AI Integration

When working with AI coding assistants, reference the change:

"Please help me implement change `add-user-auth`. The tasks are in `.super-dev/changes/add-user-auth/tasks.md` and the specs are in `.super-dev/changes/add-user-auth/specs/`."
