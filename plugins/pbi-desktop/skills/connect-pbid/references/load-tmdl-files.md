# Loading TMDL / BIM Files into TOM

Load a local semantic model definition (TMDL folder or BIM file) into TOM for programmatic inspection and modification; no running Analysis Services instance required.

## Via te CLI (recommended; cross-platform)

The `te` CLI loads TMDL and BIM natively on macOS, Linux, and Windows. It wraps TOM internally and provides a complete model CRUD surface.

### Load and inspect

```bash
# Load TMDL folder
te load -m ./MyModel.SemanticModel/definition

# Load BIM file
te load -m ./model.bim

# List tables and measures
te ls -m ./MyModel.SemanticModel/definition

# List columns in a specific table
te ls -m ./MyModel.SemanticModel/definition Sales

# Read a measure expression
te cat "Sales/Total Revenue" -m ./MyModel.SemanticModel/definition

# Get object properties as JSON
te get "Sales/Total Revenue" -m ./MyModel.SemanticModel/definition
```

### Modify and save

```bash
# Add a measure
te add "Sales/YTD Revenue.measure" -m ./definition -i "TOTALYTD([Total Revenue], 'Date'[Date])" --save

# Rename an object
te mv "Sales/Old Name" "Sales/New Name" -m ./definition --save

# Set a property
te set "Sales/Total Revenue" -m ./definition -q description -i "Sum of revenue across all channels" --save

# Remove an object
te rm "Sales/Deprecated Measure" -m ./definition --save

# Save to a different location or format
te save -m ./definition -o ./export --format bim
te save -m ./model.bim -o ./tmdl-out --format tmdl
```

### Deploy to Fabric

```bash
# Deploy local TMDL to a remote workspace
te deploy -m ./definition -s "My Workspace" -d "My Model" --auth interactive
```

### Connect to remote model and save locally

```bash
# Pull a remote model to local TMDL
te save -s "My Workspace" -d "My Model" -o ./local-copy --format tmdl --auth interactive

# Or export via fab CLI
fab export "My Workspace.Workspace/My Model.SemanticModel" -o ./local-copy -f
```

## Via PowerShell + TOM (Windows)

On Windows, load TMDL/BIM directly via the TOM .NET assemblies. This avoids any running AS instance; TOM deserializes the files into an in-memory Database object.

### Prerequisites

TOM NuGet package installed at `$env:TEMP\tom_nuget`:

```powershell
$pkgDir = "$env:TEMP\tom_nuget"
if (-not (Test-Path "$pkgDir\Microsoft.AnalysisServices.retail.amd64\lib\net45\Microsoft.AnalysisServices.Tabular.dll")) {
    nuget install Microsoft.AnalysisServices.retail.amd64 -OutputDirectory $pkgDir -ExcludeVersion | Out-Null
}
Add-Type -Path "$pkgDir\Microsoft.AnalysisServices.retail.amd64\lib\net45\Microsoft.AnalysisServices.Core.dll"
Add-Type -Path "$pkgDir\Microsoft.AnalysisServices.retail.amd64\lib\net45\Microsoft.AnalysisServices.Tabular.dll"
Add-Type -Path "$pkgDir\Microsoft.AnalysisServices.retail.amd64\lib\net45\Microsoft.AnalysisServices.Tabular.Json.dll"
```

### Load TMDL folder

```powershell
$tmdlPath = "C:\Projects\MyModel.SemanticModel\definition"
$db = [Microsoft.AnalysisServices.Tabular.TmdlSerializer]::DeserializeDatabaseFromFolder($tmdlPath)
$model = $db.Model

Write-Output "Loaded: $($db.Name) (compat $($db.CompatibilityLevel))"
Write-Output "Tables: $($model.Tables.Count)"
```

### Load BIM file

```powershell
$bimPath = "C:\Projects\model.bim"
$json = [System.IO.File]::ReadAllText($bimPath)
$db = [Microsoft.AnalysisServices.Tabular.JsonSerializer]::DeserializeDatabase($json)
$model = $db.Model

Write-Output "Loaded: $($db.Name) (compat $($db.CompatibilityLevel))"
Write-Output "Tables: $($model.Tables.Count)"
```

### Inspect the loaded model

Once loaded, the `$model` object has the same TOM API as a live connection:

```powershell
# List tables
foreach ($table in $model.Tables) {
    Write-Output "$($table.Name): $($table.Columns.Count) cols, $($table.Measures.Count) measures"
}

# Read a measure
$m = $model.Tables["Sales"].Measures["Total Revenue"]
Write-Output "$($m.Name) = $($m.Expression)"

# List relationships
foreach ($rel in $model.Relationships) {
    $r = [Microsoft.AnalysisServices.Tabular.SingleColumnRelationship]$rel
    Write-Output "$($r.FromTable.Name)[$($r.FromColumn.Name)] -> $($r.ToTable.Name)[$($r.ToColumn.Name)]"
}
```

### Modify and save back

```powershell
# Add a measure
$measure = New-Object Microsoft.AnalysisServices.Tabular.Measure
$measure.Name = "YTD Revenue"
$measure.Expression = "TOTALYTD([Total Revenue], 'Date'[Date])"
$model.Tables["Sales"].Measures.Add($measure)

# Save back to TMDL
[Microsoft.AnalysisServices.Tabular.TmdlSerializer]::SerializeDatabaseToFolder($db, $tmdlPath)

# Or save as BIM
$json = [Microsoft.AnalysisServices.Tabular.JsonSerializer]::SerializeDatabase($db)
[System.IO.File]::WriteAllText("C:\export\model.bim", $json)
```

### Deploy the modified model

After modifying the in-memory model, deploy to a remote workspace:

```powershell
# Option 1: save to TMDL, then use fab import
[Microsoft.AnalysisServices.Tabular.TmdlSerializer]::SerializeDatabaseToFolder($db, $tmdlPath)
fab import "WorkspaceName.Workspace/ModelName.SemanticModel" -i $tmdlPath -f

# Option 2: deploy via te CLI
te deploy -m $tmdlPath -s "WorkspaceName" -d "ModelName" --auth interactive
```

## Key Differences: Live Connection vs Local Files

| | Live connection (localhost) | Local files (TMDL/BIM) |
|---|---|---|
| **Source** | Running `msmdsrv.exe` process | Files on disk |
| **SaveChanges** | Writes to AS engine instantly | Must serialize back to disk |
| **Refresh** | Can trigger data refresh | No data; schema only |
| **DAX queries** | Yes (via ADOMD.NET) | No (no engine running) |
| **DMV queries** | Yes | No |
| **Undo** | `UndoLocalChanges()` discards unsaved | Revert files via git |
| **Deploy** | Already live | Needs `fab import` or `te deploy` |

## Common Patterns

### Round-trip: pull, modify, push

```bash
# Pull from Fabric
fab export "Prod.Workspace/Sales.SemanticModel" -o ./working -f

# Modify locally
te add "Sales/New KPI.measure" -m ./working/Sales.SemanticModel/definition \
  -i "DIVIDE([Revenue], [Target])" --save

# Push back
fab import "Prod.Workspace/Sales.SemanticModel" -i ./working/Sales.SemanticModel -f
```

### Convert between formats

```bash
# BIM to TMDL
te save -m ./model.bim -o ./tmdl-output --format tmdl

# TMDL to BIM
te save -m ./definition -o ./output --format bim

# TMDL to PBIP (full project with report stub)
te save -m ./definition -o ./project --format pbip
```
