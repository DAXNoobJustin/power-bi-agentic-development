// tmdl-validate: Structural validator for TMDL files
//
// Validates indent structure, keyword recognition, property syntax,
// backtick quoting, and parent-child nesting rules.
// Does NOT validate DAX/M expressions or cross-file references.

use std::env;
use std::fs;
use std::process;


// #region Types

#[derive(Debug, Clone, Copy, PartialEq)]
enum Severity {
    Error,
    Warning,
}

struct Diagnostic {
    line: usize,
    severity: Severity,
    message: String,
}

struct ObjectDef {
    keyword: &'static str,
    allowed_parents: &'static [&'static str],
}

// #endregion


// #region Schema

const ROOT_OBJECTS: &[&str] = &[
    "model",
    "database",
    "table",
    "relationship",
    "role",
    "cultureInfo",
    "perspective",
    "dataSource",
    "expression",
    "queryGroup",
    "annotation",
    "extendedProperty",
    "ref",
];

const NESTED_OBJECTS: &[ObjectDef] = &[
    ObjectDef { keyword: "column", allowed_parents: &["table"] },
    ObjectDef { keyword: "measure", allowed_parents: &["table"] },
    ObjectDef { keyword: "hierarchy", allowed_parents: &["table"] },
    ObjectDef { keyword: "level", allowed_parents: &["hierarchy"] },
    ObjectDef { keyword: "partition", allowed_parents: &["table"] },
    ObjectDef { keyword: "calculationGroup", allowed_parents: &["table"] },
    ObjectDef { keyword: "calculationItem", allowed_parents: &["calculationGroup"] },
    ObjectDef { keyword: "annotation", allowed_parents: &["table", "column", "measure", "partition", "hierarchy", "level", "calculationItem", "calculationGroup", "role", "perspective", "cultureInfo", "dataSource", "expression", "relationship", "model", "database", "queryGroup", "function", "tablePermission", "columnPermission", "perspectiveTable", "perspectiveColumn", "perspectiveMeasure", "perspectiveHierarchy", "linguisticMetadata", "member"] },
    ObjectDef { keyword: "extendedProperty", allowed_parents: &["table", "column", "measure", "partition", "hierarchy", "level", "calculationItem", "calculationGroup", "role", "perspective", "cultureInfo", "dataSource", "expression", "relationship", "model", "database", "queryGroup", "function", "tablePermission", "columnPermission", "perspectiveTable", "perspectiveColumn", "perspectiveMeasure", "perspectiveHierarchy", "linguisticMetadata", "member"] },
    ObjectDef { keyword: "tablePermission", allowed_parents: &["role"] },
    ObjectDef { keyword: "columnPermission", allowed_parents: &["tablePermission"] },
    ObjectDef { keyword: "perspectiveTable", allowed_parents: &["perspective"] },
    ObjectDef { keyword: "perspectiveColumn", allowed_parents: &["perspectiveTable"] },
    ObjectDef { keyword: "perspectiveMeasure", allowed_parents: &["perspectiveTable"] },
    ObjectDef { keyword: "perspectiveHierarchy", allowed_parents: &["perspectiveTable"] },
    ObjectDef { keyword: "linguisticMetadata", allowed_parents: &["cultureInfo"] },
    ObjectDef { keyword: "translation", allowed_parents: &["cultureInfo"] },
    ObjectDef { keyword: "dataAccessOptions", allowed_parents: &["model"] },
    ObjectDef { keyword: "ref", allowed_parents: &["model", "table"] },
    ObjectDef { keyword: "formatStringDefinition", allowed_parents: &["measure", "calculationItem"] },
    ObjectDef { keyword: "detailRowsDefinition", allowed_parents: &["measure", "table"] },
    ObjectDef { keyword: "kpiStatusExpression", allowed_parents: &["measure"] },
    ObjectDef { keyword: "kpiTargetExpression", allowed_parents: &["measure"] },
    ObjectDef { keyword: "kpiTrendExpression", allowed_parents: &["measure"] },
    ObjectDef { keyword: "alternateOf", allowed_parents: &["column"] },
];

const KNOWN_PROPERTIES: &[&str] = &[
    "lineageTag", "isHidden", "description", "displayFolder",
    "culture", "defaultPowerBIDataSourceVersion", "discourageImplicitMeasures", "sourceQueryCulture",
    "compatibilityLevel",
    "dataCategory", "isPrivate", "showAsVariationsOnly", "excludeFromModelRefresh", "sourceLineageTag",
    "dataType", "formatString", "summarizeBy", "sourceColumn", "sortByColumn",
    "isNameInferred", "isDefaultLabel", "isDefaultImage", "isKey", "isNullable", "isUnique",
    "isAvailableInMdx", "encodingHint", "keepUniqueRows", "columnType", "isDataTypeInferred", "alignment",
    "expression", "homeMeasure",
    "fromColumn", "toColumn", "crossFilteringBehavior", "isActive", "joinOnDateBehavior",
    "relyOnReferentialIntegrity", "securityFilteringBehavior", "fromCardinality", "toCardinality",
    "mode", "source", "type",
    "modelPermission", "filterExpression",
    "ordinal", "precedence",
    "legacyRedirects", "returnErrorValuesAsNull", "fastCombine",
    "targetProperty", "value",
    "column", "content",
    "queryGroup",
];

fn all_known_keywords() -> Vec<&'static str> {
    let mut kws: Vec<&str> = ROOT_OBJECTS.to_vec();
    for obj in NESTED_OBJECTS {
        if !kws.contains(&obj.keyword) {
            kws.push(obj.keyword);
        }
    }
    kws.push("function");
    kws.push("member");
    kws
}

const KNOWN_DATA_TYPES: &[&str] = &[
    "string", "int64", "double", "datetime", "boolean", "decimal", "binary", "unknown",
];

const KNOWN_SUMMARIZE_BY: &[&str] = &[
    "sum", "count", "min", "max", "average", "distinctcount", "none", "default",
];

// #endregion


// #region Validation

fn validate_tmdl(content: &str, filename: &str) -> Vec<Diagnostic> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut context_stack: Vec<(&str, usize)> = Vec::new();
    let mut in_expression_block = false;
    let mut in_backtick_block = false;
    let mut expression_indent: usize = 0;
    let mut prev_indent: usize = 0;
    let mut is_first_content_line = true;
    let mut pending_expression = false; // next line starts an expression block

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        if line.trim().is_empty() {
            continue;
        }

        // Handle triple-backtick expression blocks
        if in_backtick_block {
            if line.trim().starts_with("```") {
                in_backtick_block = false;
            }
            continue;
        }

        // Check for space indentation
        if !in_expression_block && line.starts_with("  ") {
            diagnostics.push(Diagnostic {
                line: line_num,
                severity: Severity::Error,
                message: "spaces used for indentation; TMDL requires tabs".into(),
            });
            continue;
        }

        let indent = line.len() - line.trim_start_matches('\t').len();
        let trimmed = line.trim_start_matches('\t');

        // Enter expression block if previous line ended with `=`
        if pending_expression {
            pending_expression = false;
            in_expression_block = true;
            // expression_indent was set on the previous line
        }

        // Handle expression blocks (DAX, M, JSON)
        if in_expression_block {
            if indent <= expression_indent && !trimmed.is_empty() {
                in_expression_block = false;
            } else {
                continue;
            }
        }

        // Check for triple-backtick opening on this line
        if trimmed.contains("```") {
            let count = trimmed.matches("```").count();
            if count == 1 {
                if !trimmed.starts_with("```") {
                    // Opening backtick at end of declaration line
                    in_backtick_block = true;
                }
            }
            // count == 2 means self-contained open+close on one line
        }

        // Check indent jump
        if !is_first_content_line && indent > prev_indent + 1 {
            diagnostics.push(Diagnostic {
                line: line_num,
                severity: Severity::Error,
                message: format!(
                    "indent jumps from level {} to {} (max increase is 1)",
                    prev_indent, indent
                ),
            });
        }

        // Doc comment
        if trimmed.starts_with("///") {
            prev_indent = indent;
            is_first_content_line = false;
            continue;
        }

        let first_word = trimmed.split(|c: char| c.is_whitespace() || c == '\'').next().unwrap_or("");

        let is_root_object = ROOT_OBJECTS.contains(&first_word);
        let nested_def = NESTED_OBJECTS.iter().find(|o| o.keyword == first_word);
        let is_object = is_root_object || nested_def.is_some();

        if is_object {
            // Pop context stack to current indent
            while let Some((_, ctx_indent)) = context_stack.last() {
                if *ctx_indent >= indent {
                    context_stack.pop();
                } else {
                    break;
                }
            }

            // Validate nesting
            if let Some(def) = nested_def {
                if let Some((parent_kw, parent_indent)) = context_stack.last() {
                    if !def.allowed_parents.contains(parent_kw) {
                        diagnostics.push(Diagnostic {
                            line: line_num,
                            severity: Severity::Error,
                            message: format!(
                                "'{}' cannot be nested inside '{}' (allowed: {})",
                                first_word, parent_kw, def.allowed_parents.join(", ")
                            ),
                        });
                    }
                    // Nested object must be exactly one indent deeper than parent
                    if indent != parent_indent + 1 {
                        diagnostics.push(Diagnostic {
                            line: line_num,
                            severity: Severity::Error,
                            message: format!(
                                "'{}' at indent {} but parent '{}' is at indent {} (expected {})",
                                first_word, indent, parent_kw, parent_indent, parent_indent + 1
                            ),
                        });
                    }
                }
            }

            // Root-only objects at wrong indent
            if is_root_object && nested_def.is_none() && indent != 0 {
                diagnostics.push(Diagnostic {
                    line: line_num,
                    severity: Severity::Error,
                    message: format!("'{}' must be at indent level 0 (found at level {})", first_word, indent),
                });
            }

            context_stack.push((first_word, indent));

            // Check for multi-line expression (next line starts the block)
            if trimmed.ends_with(" =") || trimmed.ends_with("\t=") {
                pending_expression = true;
                expression_indent = indent;
            }
        } else if let Some(colon_pos) = trimmed.find(':') {
            // Property line
            let prop_name = trimmed[..colon_pos].trim();

            if !prop_name.is_empty()
                && !KNOWN_PROPERTIES.contains(&prop_name)
                && !prop_name.starts_with('#')
            {
                diagnostics.push(Diagnostic {
                    line: line_num,
                    severity: Severity::Warning,
                    message: format!("unknown property '{}'", prop_name),
                });
            }

            // Property must be exactly one indent deeper than parent object
            if let Some((parent_kw, parent_indent)) = context_stack.last() {
                if indent != parent_indent + 1 {
                    diagnostics.push(Diagnostic {
                        line: line_num,
                        severity: Severity::Error,
                        message: format!(
                            "property '{}' at indent {} but parent '{}' is at indent {} (expected {})",
                            prop_name, indent, parent_kw, parent_indent, parent_indent + 1
                        ),
                    });
                }
            }

            // Validate dataType value
            if prop_name == "dataType" {
                let value = trimmed[colon_pos + 1..].trim().to_lowercase();
                if !value.is_empty() && !KNOWN_DATA_TYPES.contains(&value.as_str()) {
                    diagnostics.push(Diagnostic {
                        line: line_num,
                        severity: Severity::Error,
                        message: format!("unknown dataType '{}' (expected: {})", value, KNOWN_DATA_TYPES.join(", ")),
                    });
                }
            }

            // Validate summarizeBy value
            if prop_name == "summarizeBy" {
                let value = trimmed[colon_pos + 1..].trim().to_lowercase();
                if !value.is_empty() && !KNOWN_SUMMARIZE_BY.contains(&value.as_str()) {
                    diagnostics.push(Diagnostic {
                        line: line_num,
                        severity: Severity::Error,
                        message: format!("unknown summarizeBy '{}' (expected: {})", value, KNOWN_SUMMARIZE_BY.join(", ")),
                    });
                }
            }
        } else {
            // Boolean flag property (no colon) like `isHidden` or `discourageImplicitMeasures`
            let is_flag = KNOWN_PROPERTIES.contains(&first_word);

            if is_flag {
                // Flag must be one indent deeper than parent
                if let Some((parent_kw, parent_indent)) = context_stack.last() {
                    if indent != parent_indent + 1 {
                        diagnostics.push(Diagnostic {
                            line: line_num,
                            severity: Severity::Error,
                            message: format!(
                                "flag '{}' at indent {} but parent '{}' is at indent {} (expected {})",
                                first_word, indent, parent_kw, parent_indent, parent_indent + 1
                            ),
                        });
                    }
                }
            } else if !first_word.is_empty()
                && !first_word.starts_with("//")
                && !all_known_keywords().contains(&first_word)
            {
                // Unknown word that isn't a keyword, property, or comment
                diagnostics.push(Diagnostic {
                    line: line_num,
                    severity: Severity::Error,
                    message: format!("unrecognized keyword '{}'", first_word),
                });
            }
        }

        // Catch any line ending with `=` as a multi-line expression start
        // (covers property-style like `source =` that don't match as objects)
        if !pending_expression && (trimmed.ends_with(" =") || trimmed.ends_with("\t=") || trimmed == "=") {
            pending_expression = true;
            expression_indent = indent;
        }

        prev_indent = indent;
        is_first_content_line = false;
    }

    // File-level checks
    if filename.ends_with("database.tmdl") {
        if !lines.iter().any(|l| l.trim_start().starts_with("database")) {
            diagnostics.push(Diagnostic {
                line: 1, severity: Severity::Error,
                message: "database.tmdl must contain a 'database' declaration".into(),
            });
        }
    }

    if filename.ends_with("model.tmdl") {
        if !lines.iter().any(|l| l.trim_start().starts_with("model")) {
            diagnostics.push(Diagnostic {
                line: 1, severity: Severity::Error,
                message: "model.tmdl must contain a 'model' declaration".into(),
            });
        }
    }

    diagnostics
}

// #endregion


// #region Main

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("tmdl-validate 0.1.0");
        eprintln!("Structural validator for TMDL files");
        eprintln!();
        eprintln!("Usage: tmdl-validate <file.tmdl> [--json]");
        process::exit(1);
    }

    let filepath = &args[1];
    let json_output = args.iter().any(|a| a == "--json");

    let content = match fs::read_to_string(filepath) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", filepath, e);
            process::exit(1);
        }
    };

    let diagnostics = validate_tmdl(&content, filepath);

    let errors: Vec<&Diagnostic> = diagnostics.iter().filter(|d| d.severity == Severity::Error).collect();
    let warnings: Vec<&Diagnostic> = diagnostics.iter().filter(|d| d.severity == Severity::Warning).collect();

    if json_output {
        print!("[");
        for (i, d) in diagnostics.iter().enumerate() {
            if i > 0 { print!(","); }
            print!(
                "{{\"line\":{},\"severity\":\"{}\",\"message\":\"{}\"}}",
                d.line,
                match d.severity { Severity::Error => "error", Severity::Warning => "warning" },
                d.message.replace('"', "\\\"")
            );
        }
        println!("]");
    } else {
        for d in &diagnostics {
            let prefix = match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warn ",
            };
            eprintln!("{}:{}  {} {}", filepath, d.line, prefix, d.message);
        }

        if errors.is_empty() && warnings.is_empty() {
            eprintln!("Valid TMDL (0 errors, 0 warnings)");
        } else {
            eprintln!();
            eprintln!("{} error(s), {} warning(s)", errors.len(), warnings.len());
        }
    }

    if !errors.is_empty() {
        process::exit(2);
    }
}

// #endregion
