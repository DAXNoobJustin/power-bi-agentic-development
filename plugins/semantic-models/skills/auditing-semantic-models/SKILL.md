---
name: auditing-semantic-models
version: 0.10.0
description: This skill should be used when the user asks to "audit a semantic model", "check model quality", "optimize my model", "validate model design", "run a best practice check", or mentions auditing or validating a Power BI semantic model against quality or best practice standards.
---

# Auditing Semantic Models

Prescriptive workflow for auditing Power BI semantic models against quality, performance, and best practice standards using TMDL analysis.

## Workflow

### 1. Export the Model

Export the semantic model definition to a local directory for analysis:

```bash
mkdir -p /tmp/audit
fab export "Workspace.Workspace/Model.SemanticModel" -o /tmp/audit -f
```

### 2. Analyze TMDL Structure

Read the exported TMDL files:

```
/tmp/audit/Model.SemanticModel/
  definition/
    model.tmdl           # Model-level settings
    database.tmdl        # Database config
    tables/              # Table definitions (*.tmdl)
    relationships.tmdl   # Relationships
    expressions.tmdl     # M expressions (if present)
```

### 3. Audit Categories

Evaluate findings across five categories, ordered by severity:

**Critical**
- Bidirectional relationships (ambiguity risk)
- Circular dependencies between tables
- Missing data types on columns
- Tables without relationships (orphaned)

**Memory and Size**
- High-cardinality columns with large dictionaries (GUIDs, transaction IDs, composite keys)
- Unsplit DateTime columns (near-unique precision creating massive dictionaries)
- Attribute hierarchies (IsAvailableInMDX) enabled on hidden or high-cardinality columns
- Auto Date/Time tables (hidden LocalDateTable_* bloating memory)
- Inappropriate data types (Double for currency, String for numeric)
- Calculated columns that could be measures
- Unused columns or tables (no references in measures or visuals)
- DISTINCTCOUNT on high-cardinality columns without optimization

**Data Reduction**
- Unfiltered history in fact tables (no date-range filter or incremental refresh)
- DAX calculated columns that could be Power Query computed columns
- Pre-summarization opportunities (detail grain not needed for reporting)

**DAX Anti-Patterns**
- Nested CALCULATE (unnecessary complexity)
- Division without DIVIDE() or error handling
- Inefficient iterators (SUMX/AVERAGEX over large tables without filters)
- ALL() where REMOVEFILTERS() is more appropriate

**Documentation**
- Tables or columns missing descriptions
- Missing display folders for measures
- Inconsistent naming conventions (mixed case, abbreviations)

**Design**
- Star schema violations (direct fact-to-fact relationships, snowflake patterns)
- Missing or misconfigured date table (no `isDateTable` mark)
- Excessive columns per table (>30 suggests denormalization issues)
- Many-to-many relationships without bridging tables

**Direct Lake (if applicable)**
- Delta table health (parquet file count, V-Order, row group sizes)
- DirectQuery fallback risk (RLS definitions, SQL endpoint views)

**AI and Copilot Readiness** (informational, not issues)
- Duplicate field names across tables (confuses Copilot/data agents)
- Complex patterns (disconnected tables, many-to-many, inactive relationships) are valid model design but AI may not be the right tool for querying those areas

### 4. Report Findings

Produce a structured markdown report with:

- Summary table of finding counts by severity
- Detailed findings with file locations and line numbers where possible
- Specific remediation recommendations for each finding
- Prioritized action list (critical first)

## Using the Semantic Model Auditor Agent

Dispatch the `semantic-model-auditor` agent to perform the audit. The agent handles export, analysis, and reporting autonomously.

## Notes

- The audit analyzes TMDL metadata only -- it does not execute DAX queries or check data quality
- For DAX query performance testing, use `fab api -A powerbi` to run test queries after the structural audit
- For BPA rule-based analysis with Tabular Editor, see the `bpa-rules` skill in the tabular-editor plugin
