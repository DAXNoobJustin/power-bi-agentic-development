# Theme Compliance

A Power BI report is "theme-compliant" when its visuals inherit formatting from the theme rather than carrying redundant or conflicting bespoke overrides in their `visual.json` files. Non-compliance accumulates through manual formatting in Power BI Desktop, copy-pasted visuals, and theme switches where old overrides were never cleared.

---

## What Counts as a Violation

Not all bespoke overrides are violations. Classify each override in a visual as:

| Category | Description | Action |
|----------|-------------|--------|
| **Stale** | Duplicates what the theme already sets with the same value | Remove — it's noise and will prevent future theme changes from taking effect |
| **Conflicting** | Overrides the theme with a different value for no documented reason | Investigate — promote to theme if it should apply broadly, or document why it's an exception |
| **Intentional exception** | Legitimately differs from the theme default for a specific reason | Keep — but add an annotation comment if possible (see `pbir-format` → `annotations.md`) |
| **Conditional formatting** | Expression-based formatting in `objects` | Keep — do not clear CF expressions |

---

## Audit Workflow

### Step 1 — Establish the Theme Baseline

Before evaluating visuals, know exactly what the theme sets:

```bash
THEME_NAME=$(jq -r '.themeCollection.customTheme.name' Report.Report/definition/report.json)
THEME="Report.Report/StaticResources/RegisteredResources/$THEME_NAME"

# What does the wildcard set?
jq '.visualStyles["*"]["*"] | keys' "$THEME"

# What visual types have type-level overrides?
jq '[.visualStyles | keys[] | select(. != "*")]' "$THEME"

# For each type found above, what does the type override?
jq '.visualStyles.textbox["*"] | keys' "$THEME"
jq '.visualStyles.kpi["*"] | keys' "$THEME"
```

### Step 2 — Inventory Bespoke Overrides

Scan all visual.json files and collect which ones have overrides:

```bash
# List every visual.json that has objects or visualContainerObjects
find Report.Report/definition/pages -name "visual.json" -print0 | \
  xargs -0 -I{} sh -c \
  'HAS=$(jq -r "(.visual.objects != null) or (.visual.visualContainerObjects != null)" "{}"); \
   [ "$HAS" = "true" ] && echo "{}"'
```

For each flagged visual, extract the override inventory:

```bash
jq '{
  path: input_filename,
  visualType: .visual.visualType,
  objectKeys: (.visual.objects | keys // []),
  vcoKeys: (.visual.visualContainerObjects | keys // [])
}' visual.json
```

### Step 3 — Classify Each Override

For each key in `objectKeys` and `vcoKeys`, compare the visual's value against the theme:

```bash
# Example: compare a visual's title setting against the theme wildcard
VISUAL_TITLE=$(jq '.visual.visualContainerObjects.title[0].show' visual.json)
THEME_TITLE=$(jq '.visualStyles["*"]["*"].title[0].show' "$THEME")
echo "Visual: $VISUAL_TITLE | Theme: $THEME_TITLE"
```

If they match → stale override, safe to remove.
If they differ → investigate (intentional exception or should be promoted to theme).

### Step 4 — Act on Findings

Depending on classification:

- **Stale:** `jq 'del(.visual.visualContainerObjects.<key>)'` or use `pbir visuals clear-formatting`
- **Conflicting, should be global:** Promote to theme (see `theme-authoring.md` — Promoting Bespoke Formatting)
- **Conflicting, is an exception:** Document why via annotation or leave with a note in your scratchpad

---

## Severity Levels

### Critical

Overrides that actively break the theme or create inconsistency that users will notice:

- Container chrome (title/border/background) set to different font, color, or visibility than the theme wildcard across multiple visuals of the same type
- `dropShadow.show: true` on individual visuals when theme sets `false`
- Hardcoded hex colors in `objects.dataPoint` that don't match theme `dataColors`

**Fix:** Promote to visual-type theme override or clear the stale override.

### Warning

Overrides that create inconsistency but are less visible:

- `padding` or `border.radius` differing from theme defaults on individual visuals
- Axis label font or size differing from `textClasses.label` on individual visuals
- Legend position set per-visual when a type-level default would be cleaner

**Fix:** Evaluate whether a visual-type override in the theme is warranted. If only 1–2 visuals differ, leave as exception. If 3+, promote to theme.

### Suggestion

Low-priority cleanup:

- `visualContainerObjects` keys that exist but hold null or empty values
- `objects` keys for properties where the visual doesn't actually render that property (e.g., `categoryAxis` on a card visual)
- Redundant `general` container objects with no meaningful properties

**Fix:** Remove at next refactor pass.

---

## Common Stale Override Patterns

### Title Formatting Duplicated

The theme wildcard sets `title[0].fontSize = 14` and `title[0].fontFamily = "Segoe UI Semibold"`. A visual's `visualContainerObjects` also has `title[0].fontSize = 14`. This is stale.

**Detection:**
```bash
jq 'select(.visual.visualContainerObjects.title != null) | .visual.visualContainerObjects.title[0].fontSize' visual.json
```

### Background Explicitly Set to `false` When Theme Already Disables It

The theme wildcard sets `background[0].show = false`. A visual also has `visualContainerObjects.background[0].show = false`. This is stale noise.

### Axis Font Locked In From Before a Font Change

The theme's `textClasses.label.fontFace` was updated to Segoe UI, but 12 line charts still have `objects.categoryAxis[0].fontFamily` from a previous theme. The font doesn't update because the bespoke override wins.

**Detection:**
```bash
find Report.Report/definition/pages -name "visual.json" -print0 | \
  xargs -0 jq -r 'select(.visual.objects.categoryAxis != null) | input_filename'
```

### Shadow Left On After Theme Change

Previous theme had `dropShadow.show: true` in the wildcard. New theme sets `false`. Some visuals have `visualContainerObjects.dropShadow[0].show = true` explicitly, which overrides the new theme.

---

## Fix Decision Tree

```
Override exists in visual.json
│
├── Is it a CF expression? (conditional formatting)
│   └── YES → Keep unconditionally
│
├── Does it match the theme value exactly?
│   └── YES → Stale. Remove it.
│
├── Does it differ from the theme?
│   ├── Is this the only visual that needs this value?
│   │   └── YES → Intentional exception. Keep (and document if complex).
│   │
│   └── Do 3+ visuals of the same type need this value?
│       └── YES → Promote to theme as a visual-type override, then remove from visuals.
```

---

## Batch Clearing

When a full compliance sweep is required after a theme overhaul:

```bash
# Option A: CLI (preferred) — clears all except CF
pbir visuals clear-formatting "Report.Report/**/*.Visual" --keep-cf -f

# Option B: Clear container chrome only across all visuals
find Report.Report/definition/pages -name "visual.json" -print0 | \
  xargs -0 -I{} sh -c \
  'jq "del(.visual.visualContainerObjects)" "{}" > "{}.tmp" && mv "{}.tmp" "{}"'

# Validate all after batch operation
find Report.Report/definition/pages -name "visual.json" -print0 | \
  xargs -0 -I{} sh -c 'jq empty "{}" && echo "OK: {}"'
```

> After batch clearing, always do a visual-render check in Power BI Desktop or Service. Some visuals may have had intentional exceptions that the batch operation removed.
