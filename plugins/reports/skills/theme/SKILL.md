---
name: theme
version: 0.8.8
description: "This skill should be used when the user asks to 'create a theme', 'design a theme', 'build a theme from scratch', 'apply a theme to the report', 'enforce theme compliance', 'audit theme adherence', 'push formatting to theme', 'promote bespoke formatting to theme', 'clear visual overrides', 'make all visuals consistent with the theme', 'standardize report formatting', 'set up a theme', 'update theme colors', 'change theme typography', 'set theme text classes', 'validate a theme', 'add visual-type overrides to the theme', 'ensure visuals use theme defaults', or needs to design, enforce, audit, or validate a Power BI report theme beyond simple JSON inspection."
---

# Power BI Report Themes

Covers design, enforcement, compliance auditing, and validation of Power BI report themes. For PBIR JSON mechanics — exact property names, filter pane selectors, ThemeDataColor syntax, and `jq` patterns — see the **`pbir-format`** skill (pbip plugin) → `references/theme.md`.

> **Tooling preference:** Use `pbir` CLI when available (`pbir theme colors`, `pbir visuals clear-formatting`). Fall back to direct `jq` modification when unavailable. Always validate with `jq empty <file>` after every write.

## The Formatting Hierarchy

Power BI applies visual formatting through a four-level cascade. Each level overrides the level above it:

```
Level 1  Power BI built-in defaults
         |
Level 2  Theme wildcard     visualStyles["*"]["*"]           applies to ALL visuals
         |
Level 3  Theme visual-type  visualStyles["lineChart"]["*"]   overrides wildcard for that type
         |
Level 4  Visual instance    visual.json objects +            overrides everything
                            visualContainerObjects
```

### Core Principle

Push as much formatting as possible into levels 2 and 3. A well-designed theme means:

- Visual JSON files stay lean — no bespoke formatting noise cluttering visual.json
- Global style changes require editing one file
- New visuals automatically inherit correct defaults without manual intervention

Visual-level overrides (level 4) should exist only for true one-offs: content-specific formatting, exceptions to the visual-type default, or conditional formatting expressions.

### Diagnosing Why a Visual Looks the Way It Does

When a visual renders unexpectedly, walk up the cascade:

1. Check `visual.json` → `objects` and `visualContainerObjects` (level 4 always wins)
2. Check theme `visualStyles["<type>"]["*"]` for that visual type (level 3)
3. Check theme `visualStyles["*"]["*"]` wildcard (level 2)
4. If absent everywhere, Power BI is applying a built-in default

## Workflow: Audit Theme Compliance

Use when assessing whether a report's visuals are inheriting from the theme or have accumulated stale overrides.

**Step 1 — Locate the custom theme:**
```bash
THEME_NAME=$(jq -r '.themeCollection.customTheme.name' Report.Report/definition/report.json)
THEME="Report.Report/StaticResources/RegisteredResources/$THEME_NAME"
```

**Step 2 — Review what the theme sets at wildcard level:**
```bash
jq '.visualStyles["*"]["*"] | keys' "$THEME"
```

**Step 3 — List visual types that have type-level overrides:**
```bash
jq '.visualStyles | keys | map(select(. != "*"))' "$THEME"
```

**Step 4 — Scan all visual.json files for bespoke overrides:**
```bash
find Report.Report/definition/pages -name "visual.json" -print0 | \
  xargs -0 -I{} sh -c \
  'HAS=$(jq -r "(.visual.objects != null) or (.visual.visualContainerObjects != null)" "{}"); \
   [ "$HAS" = "true" ] && echo "{}"'
```

**Step 5 — For each flagged visual, identify what is overridden:**
```bash
jq '{visualType: .visual.visualType, objects: (.visual.objects | keys // []), vco: (.visual.visualContainerObjects | keys // [])}' visual.json
```

**Step 6 — Determine whether each override is intentional or stale** (see `references/theme-compliance.md` for severity levels and fix decision tree).

## Workflow: Enforce Theme (Clear Overrides)

After applying a new theme or making significant theme changes, stale visual-level overrides prevent the new theme from rendering correctly.

**With `pbir` CLI (preferred):**
```bash
# Clears bespoke formatting while preserving conditional formatting expressions
pbir visuals clear-formatting "Report.Report/**/*.Visual" --keep-cf -f
```

**With `jq` (manual, per-visual):**
```bash
# Safe: clear container chrome only (title, border, background, shadow, padding)
# Does NOT touch chart-specific objects or conditional formatting
jq 'del(.visual.visualContainerObjects)' visual.json > tmp && mv tmp visual.json

# Aggressive: clear everything including chart-specific overrides
# WARNING: also removes conditional formatting — only use if CF is confirmed absent
jq 'del(.visual.objects) | del(.visual.visualContainerObjects)' visual.json > tmp && mv tmp visual.json

# Always validate after
jq empty visual.json
```

> When in doubt, clear `visualContainerObjects` only. Leave `objects` unless you have confirmed no conditional formatting exists in that visual.

## Workflow: Author or Modify a Theme

When building or substantially revising a theme:

1. **Start from a valid base.** Use the SQLBI/Data Goblins theme (in `pbir-format` examples) or a [community template](https://github.com/deldersveld/PowerBI-ThemeTemplates). Do not author from an empty `{}`.

2. **Design the color system first** (`dataColors`, semantic colors, background/foreground variants). Color decisions cascade everywhere — get them right before setting anything else.

3. **Set typography** (`textClasses`) — font face and size for `title`, `header`, `label`, `callout`, `dataTitle`. Stick to Segoe UI / Segoe UI Semibold; custom fonts will not render on other users' machines.

4. **Set wildcard container defaults** (`visualStyles["*"]["*"]`): title visibility/font/size, `dropShadow.show: false`, padding, border, filter pane (`outspacePane`, `filterCard`).

5. **Add visual-type overrides** for types that differ from the wildcard — at minimum, `textbox` and `image` to suppress title/border/background/shadow.

6. **Validate** (see below), deploy, and visually verify with multiple visual types including the filter pane.

For detailed design guidance, see **`references/theme-authoring.md`**. For visual-type override patterns, see **`references/visual-type-overrides.md`**.

## Workflow: Validate a Theme

Beyond basic JSON validity, check structural completeness:

```bash
# 1. JSON syntax is valid
jq empty "$THEME" && echo "JSON valid"

# 2. Required top-level keys exist
jq '{dataColors: (.dataColors | type), visualStyles: (.visualStyles | type), textClasses: (.textClasses | type)}' "$THEME"

# 3. Wildcard section is present
jq 'if .visualStyles["*"]["*"] then "wildcard exists" else "MISSING wildcard" end' "$THEME"

# 4. All dataColors are valid 6-digit hex strings
jq '[.dataColors[] | select(test("^#[0-9A-Fa-f]{6}$") | not)]' "$THEME"
# Should return []

# 5. No visual-type section has a null value (typo guard)
jq '[.visualStyles | to_entries[] | select(.value == null) | .key]' "$THEME"
# Should return []

# 6. Check palette length — no ColorId in the theme should equal or exceed this
jq '.dataColors | length' "$THEME"
```

After JSON validation, deploy and visually verify:
- Wildcard container chrome (titles, borders, shadows) applies to all visuals
- Filter pane and filter cards render correctly on all pages
- Visual-type overrides correctly suppress the wildcard for exempt types (e.g., textboxes have no title)
- Data colors cycle correctly on multi-series charts

## Schema and Documentation

| Resource | URL |
|----------|-----|
| Official report theme JSON schema (versioned) | [microsoft/powerbi-desktop-samples — Report Theme JSON Schema](https://github.com/microsoft/powerbi-desktop-samples/tree/main/Report%20Theme%20JSON%20Schema) |
| Microsoft Learn — Use report themes in Power BI Desktop | https://learn.microsoft.com/en-us/power-bi/create-reports/desktop-report-themes |
| Community theme templates | [deldersveld/PowerBI-ThemeTemplates](https://github.com/deldersveld/PowerBI-ThemeTemplates) |
| PBIR item schemas | [microsoft/powerbi-desktop-samples — item-schemas](https://github.com/microsoft/powerbi-desktop-samples/tree/main/item-schemas) |

## Theme Top-Level Keys

| Key | Type | Purpose |
|-----|------|---------|
| `name` | string | Display name shown in Power BI UI |
| `dataColors` | string[] | Ordered hex palette for data series |
| `sentimentColors` | object | `good`, `bad`, `neutral` for CF measure returns |
| `maximum` / `center` / `minimum` | string | Gradient color extremes |
| `foreground` variants | string | `foreground`, `foregroundLight`, `foregroundDark`, `foregroundNeutral` |
| `background` variants | string | `background`, `backgroundLight`, `backgroundNeutral`, `backgroundDark` |
| `textClasses` | object | Typography per semantic role (`title`, `label`, `callout`, `header`, etc.) |
| `visualStyles` | object | `[visualType][state]` formatting cascade |

## References

- **`references/theme-authoring.md`** — Color system design, typography, wildcard minimum set, promoting bespoke visual.json formatting to theme level
- **`references/theme-compliance.md`** — Systematic audit workflow, stale override classification, severity levels, fix decision tree
- **`references/visual-type-overrides.md`** — Override patterns for textbox, image, shape, card, kpi, slicer, lineChart, barChart, tableEx, and matrix

## Related Skills

- **`pbir-format`** (pbip plugin) — Full theme mechanics: ThemeDataColor syntax, filter pane selectors, jq modification patterns, clearing overrides
- **`pbi-report-design`** — Report design principles: 3-30-300 rule, layout, spacing, color usage, accessibility
