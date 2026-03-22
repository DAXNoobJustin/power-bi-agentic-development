# PBIP File Types Reference

Structure and purpose of each file type in a Power BI Project (PBIP).

> Verify against the current Microsoft docs for the latest schema versions:
> - [PBIP overview](https://learn.microsoft.com/en-us/power-bi/developer/projects/projects-overview)
> - [PBIP semantic model](https://learn.microsoft.com/en-us/power-bi/developer/projects/projects-dataset)


## .pbip (Project Entry Point)

The root file that references the report folder. One per project:

```json
{
  "version": "1.0",
  "artifacts": [
    {
      "report": {
        "path": "My Report.Report"
      }
    }
  ],
  "settings": {
    "enableAutoRecovery": true
  }
}
```

When forking a project, update the `path` to match the renamed `.Report/` folder.


## .platform (Item Metadata)

Found inside each `.Report/` and `.SemanticModel/` folder. Contains the item's display name, type, and a `logicalId` used by Fabric for identity:

```json
{
  "version": "1.0",
  "$schema": "https://developer.microsoft.com/json-schemas/fabric/gitIntegration/platformProperties/2.0.0/schema.json",
  "metadata": {
    "type": "SemanticModel",
    "displayName": "SpaceParts OTC Full"
  },
  "config": {
    "logicalId": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
  }
}
```

**Critical rules:**
- `logicalId` is the unique identity of the item in Fabric. **Never change it** on an existing item -- this would break the connection to the deployed item.
- When **forking** (creating a copy), `logicalId` **must** be changed to a new GUID to avoid conflicts with the original item.
- `displayName` is what appears in the Fabric workspace. Update it when forking to distinguish the copy.
- `type` values: `SemanticModel`, `Report`


## definition.pbir (Report Entry Point)

Found inside the `.Report/` folder. References the semantic model the report is connected to:

### byPath (Local Reference)

```json
{
  "$schema": "https://developer.microsoft.com/json-schemas/fabric/item/report/definitionProperties/2.0.0/schema.json",
  "version": "4.0",
  "datasetReference": {
    "byPath": {
      "path": "../My Model.SemanticModel"
    }
  }
}
```

The `path` is relative to the `.Report/` folder. `../` navigates up to the project root.

### byConnection (Remote Model)

For reports connected to a remote semantic model (not in the same PBIP project):

```json
{
  "$schema": "https://developer.microsoft.com/json-schemas/fabric/item/report/definitionProperties/2.0.0/schema.json",
  "version": "4.0",
  "datasetReference": {
    "byConnection": {
      "connectionString": "Data Source=powerbi://api.powerbi.com/v1.0/myorg/WorkspaceName;Initial Catalog=ModelName",
      "pbiServiceModelId": null,
      "pbiModelVirtualServerName": "sobe_wowvirtualserver",
      "pbiModelDatabaseName": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "name": "EntityDataSource",
      "connectionType": "pbiServiceXmlaStyleLive"
    }
  }
}
```

### version Property

The `version` property determines format support:
- `"1.0"` -- PBIR-Legacy format only (`report.json` in the `.Report/` root)
- `"4.0"` -- PBIR format (individual `visual.json` files under `definition/pages/`)


## definition.pbism (Semantic Model Entry Point)

Found inside the `.SemanticModel/` folder:

```json
{
  "$schema": "https://developer.microsoft.com/json-schemas/fabric/item/semanticModel/definitionProperties/1.0.0/schema.json",
  "version": "4.2",
  "settings": {}
}
```

The `version` property determines the model format:
- Models with `definition/` subfolder containing `.tmdl` files use TMDL format
- Models with a `model.bim` file use TMSL/JSON format (legacy)


## visual.json (Visual Definition)

Found under `definition/pages/<pageId>/visuals/<visualId>/`. Contains the visual type, data bindings, and formatting:

```json
{
  "name": "a1b2c3d4e5f6",
  "visual": {
    "visualType": "barChart",
    "query": {
      "Commands": [
        {
          "SemanticModel": {
            "From": [
              { "Name": "d", "Entity": "Date", "Type": 0 },
              { "Name": "s", "Entity": "Sales", "Type": 0 }
            ],
            "Select": [
              {
                "Column": {
                  "Expression": { "SourceRef": { "Source": "d" } },
                  "Property": "Year"
                },
                "Name": "Date.Year"
              },
              {
                "Measure": {
                  "Expression": { "SourceRef": { "Source": "s" } },
                  "Property": "Total Revenue"
                },
                "Name": "Sales.Total Revenue"
              }
            ]
          }
        }
      ]
    }
  }
}
```

Key patterns:
- `Entity` is the table name -- update when renaming tables
- `Property` is the column or measure name -- update when renaming columns/measures
- `Name` uses `Table.ColumnOrMeasure` format (`queryRef`) -- update both parts
- `nativeQueryRef` (if present) contains only the column/measure name without table prefix -- update only when renaming columns/measures, not tables

For detailed patterns including SparklineData selectors, see [report-json-patterns.md](./report-json-patterns.md).


## page.json (Page Metadata)

Found under `definition/pages/<pageId>/`:

```json
{
  "name": "a1b2c3d4e5f6",
  "displayName": "Sales Overview",
  "displayOption": 1,
  "height": 720,
  "width": 1280
}
```

Does not contain Entity/Property references -- no updates needed during renames.


## reportExtensions.json (Report-Scoped Measures)

Found under `definition/`. Contains measures defined at the report level (not in the semantic model):

```json
{
  "entities": {
    "Sales": {
      "measures": [
        {
          "name": "Report Total",
          "expression": "SUM(Sales[Amount])"
        }
      ]
    }
  }
}
```

The `entities` keys are table names -- update when renaming tables. Measure `name` values update when renaming measures.


## semanticModelDiagramLayout.json

Found in the `.Report/` folder. Contains diagram node positions keyed by table name:

```json
{
  "version": "1.0",
  "diagrams": [
    {
      "nodes": {
        "Sales": { "x": 100, "y": 200 },
        "Date": { "x": 300, "y": 200 }
      }
    }
  ]
}
```

The `nodes` keys are table names -- update when renaming tables.
