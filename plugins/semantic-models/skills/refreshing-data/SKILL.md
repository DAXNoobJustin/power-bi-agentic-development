---
name: refreshing-data
version: 0.10.0
description: This skill should be used when the user asks to "refresh a semantic model", "trigger a dataset refresh", "check refresh status", "monitor refresh history", "schedule a refresh", or mentions refreshing data, refresh failures, or refresh monitoring for Power BI semantic models.
---

# Refreshing Data

Trigger, monitor, and troubleshoot semantic model refreshes via the Power BI REST API.

## Trigger a Refresh

### 1. Extract IDs

```bash
WS_ID=$(fab get "WorkspaceName.Workspace" -q "id" | tr -d '"')
MODEL_ID=$(fab get "WorkspaceName.Workspace/ModelName.SemanticModel" -q "id" | tr -d '"')
```

### 2. Trigger

```bash
fab api -A powerbi "groups/$WS_ID/datasets/$MODEL_ID/refreshes" -X post -i '{"type":"Full"}'
```

Refresh types:

- `Full` -- full refresh of all tables
- `Automatic` -- incremental refresh if configured, otherwise full

### 3. Monitor Status

```bash
fab api -A powerbi "groups/$WS_ID/datasets/$MODEL_ID/refreshes?\$top=1"
```

Status values: `Unknown`, `Completed`, `Failed`, `Disabled`, `InProgress`

## Quick One-Liner

```bash
WS="WorkspaceName" MODEL="ModelName" && \
WS_ID=$(fab get "$WS.Workspace" -q "id" | tr -d '"') && \
MODEL_ID=$(fab get "$WS.Workspace/$MODEL.SemanticModel" -q "id" | tr -d '"') && \
fab api -A powerbi "groups/$WS_ID/datasets/$MODEL_ID/refreshes" -X post -i '{"type":"Full"}'
```

## Check Refresh History

```bash
fab api -A powerbi "groups/$WS_ID/datasets/$MODEL_ID/refreshes?\$top=5"
```

## Notes

- Requires workspace contributor or higher permissions
- Enhanced refresh (with `commitMode`, `maxParallelism`) requires Premium/Fabric capacity
