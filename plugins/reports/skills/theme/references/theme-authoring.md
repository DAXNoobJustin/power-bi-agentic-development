# Theme Authoring Guide

Design guidance for creating and evolving Power BI report themes. This covers decisions and structure — for JSON mechanics, jq patterns, and filter pane properties, see `pbir-format` skill → `references/theme.md`.

## Starting Point

Never author a theme from an empty object. Start from:

1. **SQLBI/Data Goblins theme** — in `pbir-format` examples at `examples/K201-MonthSlicer.Report/StaticResources/RegisteredResources/SqlbiDataGoblinTheme.json`. Validated, complete, follows best practices.
2. **Community templates** — [deldersveld/PowerBI-ThemeTemplates](https://github.com/deldersveld/PowerBI-ThemeTemplates) has snippets for individual visual types.
3. **Existing report theme** — export from Power BI Service via View → Themes → Save current theme, then extend.

Then check which version of the theme schema to target:

```bash
# Schema versions are in the filename: reportThemeSchema-2.130.json
# Target the latest available at: https://github.com/microsoft/powerbi-desktop-samples/tree/main/Report%20Theme%20JSON%20Schema
```

---

## Color System Design

The color system in a theme has four layers. Design them in this order:

### 1. Data Colors (`dataColors`)

The primary series palette — ordered by expected usage frequency (most-used color first).

Rules:
- 6–12 colors recommended; fewer is more cohesive
- Colors must be visually distinguishable from each other, including for color-blind users (favor blue/orange/teal over red/green combinations for series)
- Test by listing the palette and imagining a 4-series bar chart — the first 4 colors carry the most meaning
- Muted, desaturated tones are preferable to saturated "screaming" colors

```json
"dataColors": ["#1971c2", "#f08c00", "#2f9e44", "#ae3ec9", "#e03131", "#0c8599"]
```

### 2. Semantic Colors

Used by conditional formatting measures that return color name strings (`"good"`, `"bad"`, `"neutral"`). These colors carry implied meaning and should never appear as series colors.

```json
"good": "#2f9e44",
"bad": "#e03131",
"neutral": "#868e96",
"maximum": "#1971c2",
"center": "#f8f9fa",
"minimum": "#e03131"
```

> Conditional formatting measures that return `"good"` will use whatever hex is set here. This centralizes CF color control in one place.

### 3. Background/Foreground Variants

Extended palette for container surfaces, canvas backgrounds, and foreground text. These feed into `visualContainerObjects` backgrounds and the filter pane.

```json
"foreground": "#343a40",
"foregroundLight": "#868e96",
"foregroundDark": "#212529",
"foregroundNeutralSecondary": "#adb5bd",
"background": "#ffffff",
"backgroundLight": "#f8f9fa",
"backgroundNeutral": "#e9ecef",
"backgroundDark": "#dee2e6"
```

### 4. Additional Accent Colors

```json
"tableAccent": "#1971c2",
"hyperlink": "#1971c2",
"shapeStroke": "#dee2e6",
"accent": "#1971c2"
```

### Color Principles

- Refer to `pbi-report-design` skill → `references/visual-colors.md` for WCAG contrast requirements and accessibility guidance
- Use `ThemeDataColor` references (ColorId + Percent) in theme JSON rather than hardcoded hex wherever possible — this keeps the theme internally consistent if the palette changes
- Keep `dataColors[0]` as the "primary" color that appears most frequently across the report

---

## Typography (`textClasses`)

Text classes define font properties by semantic role. Every defined class overrides Power BI's defaults for that role across all visuals.

### Standard Roles

| Role | Typical Use | Recommended Size |
|------|-------------|-----------------|
| `title` | Visual titles, page titles | 14–16pt |
| `header` | Section headers, column headers | 12–14pt |
| `label` | Axis labels, data labels | 11–12pt |
| `callout` | KPI values, prominent numbers | 28–36pt |
| `dataTitle` | KPI subtitles / labels | 12pt |
| `boldLabel` | Emphasized labels | 12pt |
| `largeTitle` | Large section titles | 20–24pt |
| `largeLabel` | Larger variant of label | 13–14pt |

### Font Choice

- Use `"'Segoe UI', wf_segoe-ui_normal, helvetica, arial, sans-serif"` for regular weight
- Use `"'Segoe UI Semibold', wf_segoe-ui_semibold, helvetica, arial, sans-serif"` for emphasis
- Do not use custom fonts — they will not render on machines where they are not installed
- Mixing more than two font weights in a report creates visual noise

### Example `textClasses` Block

```json
"textClasses": {
  "callout": {
    "fontSize": 32,
    "fontFace": "'Segoe UI', wf_segoe-ui_normal, helvetica, arial, sans-serif",
    "color": {"solid": {"color": "#343a40"}}
  },
  "title": {
    "fontSize": 14,
    "fontFace": "'Segoe UI Semibold', wf_segoe-ui_semibold, helvetica, arial, sans-serif",
    "color": {"solid": {"color": "#343a40"}}
  },
  "header": {
    "fontSize": 12,
    "fontFace": "'Segoe UI Semibold', wf_segoe-ui_semibold, helvetica, arial, sans-serif",
    "color": {"solid": {"color": "#343a40"}}
  },
  "label": {
    "fontSize": 11,
    "fontFace": "'Segoe UI', wf_segoe-ui_normal, helvetica, arial, sans-serif",
    "color": {"solid": {"color": "#495057"}}
  },
  "dataTitle": {
    "fontSize": 12,
    "fontFace": "'Segoe UI', wf_segoe-ui_normal, helvetica, arial, sans-serif",
    "color": {"solid": {"color": "#868e96"}}
  }
}
```

---

## Wildcard Container Defaults (`visualStyles["*"]["*"]`)

The wildcard section is the most important part of the theme — it sets the baseline for every visual before any type-specific overrides apply.

### Minimum Viable Wildcard

At a minimum, set:

```json
"visualStyles": {
  "*": {
    "*": {
      "title": [{
        "show": true,
        "fontSize": 14,
        "fontFamily": "'Segoe UI Semibold', wf_segoe-ui_semibold, helvetica, arial, sans-serif",
        "fontColor": {"solid": {"color": "#343a40"}},
        "background": {"solid": {"color": {"ThemeDataColor": {"ColorId": 7, "Percent": 0}}}}
      }],
      "background": [{"show": false}],
      "border": [{"show": false}],
      "dropShadow": [{"show": false}],
      "padding": [{"top": 8, "bottom": 8, "left": 8, "right": 8}]
    }
  }
}
```

### Recommended Additions

- **`subTitle`** — `show: false` by default; only specific visuals should use it
- **`divider`** — `show: false` unless design calls for it
- **`visualHeader`** — `show: true` to keep the visual header (focus mode, filter icon, etc.)
- **`outspacePane`** — filter pane styling (see `pbir-format` → `theme.md`)
- **`filterCard`** — filter card styling for Available and Applied states

### Design Guidelines

- `dropShadow.show: false` globally is strongly recommended — drop shadows create visual noise and cause vestibular issues for some users. Only enable on specific visual types that genuinely benefit.
- `background.show: false` by default keeps the canvas clean. Individual visuals can opt in.
- `border.show: false` by default — borders are clutter. Use spacing instead.
- Title should be enabled by default so visuals have useful labels. Suppress per visual type as needed (e.g., textboxes).

---

## Visual-Type Override Strategy

After setting the wildcard, add type-specific sections for visual types that need different defaults. The most critical:

| Visual Type | Why Override |
|-------------|-------------|
| `textbox` | Wildcard titles/borders don't apply to text — suppress all container chrome |
| `image` | Images rarely need a title or border |
| `shape` | Geometric shapes should have no title, background, or shadow |
| `actionButton` | Buttons have their own style system — suppress container chrome |

Less critical but commonly useful:

| Visual Type | Common Override |
|-------------|----------------|
| `kpi` | Indicator font size, trend line visibility, goal formatting |
| `card` | Category label font, value font size |
| `slicer` | Item font family/size, header font |
| `lineChart` | Legend position (`Bottom`), gridline weight |
| `tableEx` | Column header background, row alternating color |

See `references/visual-type-overrides.md` for JSON patterns for each of these.

---

## Promoting Bespoke Visual Formatting to Theme

When a `visual.json` has formatting that should apply to all visuals of its type:

1. **Identify the override:** Read the visual's `objects` (for chart properties) or `visualContainerObjects` (for container chrome)

2. **Copy to the correct theme key:**
   - `visualContainerObjects.title` → `visualStyles["<type>"]["*"].title`
   - `objects.legend` → `visualStyles["<type>"]["*"].legend`
   - `objects.categoryAxis` → `visualStyles["<type>"]["*"].categoryAxis`

3. **Apply with jq:**
   ```bash
   # Example: promote a line chart legend setting to the theme
   jq '.visualStyles.lineChart["*"].legend = [{"position": "Bottom", "show": true}]' \
     "$THEME" > "$THEME.tmp" && mv "$THEME.tmp" "$THEME"
   ```

4. **Remove the override from the visual:**
   ```bash
   jq 'del(.visual.objects.legend)' visual.json > tmp && mv tmp visual.json
   ```

5. **Validate both files:**
   ```bash
   jq empty "$THEME" && jq empty visual.json
   ```

6. **Verify the visual still renders the same way** — if it does, the promotion was successful. If it looks different, the property syntax may differ between `objects` and `visualStyles` (usually a wrapper array difference).

---

## Theme Authoring Checklist

Before considering a theme complete:

- [ ] `dataColors` has 6–12 entries; first color is the "primary"
- [ ] Semantic colors (`good`, `bad`, `neutral`) are set and distinct from series colors
- [ ] `textClasses` covers at minimum: `title`, `header`, `label`, `callout`
- [ ] Wildcard sets container defaults: `title`, `background`, `border`, `dropShadow`, `padding`
- [ ] `dropShadow.show: false` in wildcard
- [ ] At least `textbox` and `image` have type-specific overrides disabling container chrome
- [ ] Filter pane (`outspacePane` and `filterCard`) styled in wildcard
- [ ] JSON validates with `jq empty`
- [ ] Deployed and visually verified on at least 3 visual types
