use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::{BuiltinInfo, LangHandler, SymbolInfo};
use crate::util::{node_range, node_text};

pub struct OmlHandler;

impl LangHandler for OmlHandler {
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_oml::language()
    }

    fn lang_id(&self) -> &str {
        "oml"
    }

    fn extensions(&self) -> &[&str] {
        &["oml"]
    }

    fn keywords(&self) -> &[&str] {
        &[
            "name",
            "rule",
            "enable",
            "read",
            "take",
            "pipe",
            "fmt",
            "object",
            "collect",
            "match",
            "static",
            "select",
            "from",
            "where",
            "and",
            "or",
            "not",
            "in",
            "option",
            "keys",
            "get",
            "auto",
            "ip",
            "chars",
            "digit",
            "float",
            "time",
            "bool",
            "obj",
            "array",
            "time_iso",
            "time_3339",
            "time_2822",
            "time_timestamp",
            "time_clf",
            "url",
            "domain",
            "ip_net",
            "kv",
            "json",
            "base64",
        ]
    }

    fn builtins(&self) -> &[BuiltinInfo] {
        &BUILTINS
    }

    fn document_symbols(&self, tree: &Tree, src: &str) -> Vec<SymbolInfo> {
        let mut symbols = vec![];
        let root = tree.root_node();

        for i in 0..root.named_child_count() {
            let Some(child) = root.named_child(i) else {
                continue;
            };
            if child.kind() == "header" {
                for j in 0..child.named_child_count() {
                    let Some(hc) = child.named_child(j) else {
                        continue;
                    };
                    if hc.kind() == "name_field" || hc.kind() == "rule_field" {
                        if let Some(val) = hc.named_child(0) {
                            symbols.push(SymbolInfo {
                                name: node_text(&val, src).to_string(),
                                kind: if hc.kind() == "name_field" {
                                    SymbolKind::MODULE
                                } else {
                                    SymbolKind::FUNCTION
                                },
                                range: node_range(&hc),
                                selection_range: node_range(&val),
                                children: vec![],
                            });
                        }
                    }
                }
            }

            // Aggregate items as symbols
            if child.kind() == "aggregate_item" {
                collect_target_symbols(&child, src, SymbolKind::VARIABLE, &mut symbols);
            }

            // Static block items as symbols
            if child.kind() == "static_block" {
                for j in 0..child.named_child_count() {
                    let Some(si) = child.named_child(j) else {
                        continue;
                    };
                    if si.kind() == "static_item" {
                        collect_target_symbols(&si, src, SymbolKind::CONSTANT, &mut symbols);
                    }
                }
            }

            // Privacy items as symbols
            if child.kind() == "privacy_item" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    symbols.push(SymbolInfo {
                        name: node_text(&name_node, src).to_string(),
                        kind: SymbolKind::ENUM_MEMBER,
                        range: node_range(&child),
                        selection_range: node_range(&name_node),
                        children: vec![],
                    });
                }
            }
        }

        symbols
    }

    fn find_definitions(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range> {
        let mut defs = vec![];
        collect_target_defs(tree.root_node(), src, name, &mut defs);
        defs
    }

    fn find_references(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range> {
        let mut refs = vec![];
        collect_refs(tree.root_node(), src, name, &mut refs);
        refs
    }

    fn format_document(&self, _tree: &Tree, src: &str) -> Option<String> {
        // OML has a unique format with --- separator; preserve it
        Some(src.to_string())
    }
}

fn collect_target_symbols(
    node: &tree_sitter::Node,
    src: &str,
    kind: SymbolKind,
    symbols: &mut Vec<SymbolInfo>,
) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i) else {
            continue;
        };
        if child.kind() == "target" {
            if let Some(name_node) = child.child_by_field_name("name") {
                let name = node_text(&name_node, src);
                if name != "_" {
                    symbols.push(SymbolInfo {
                        name: name.to_string(),
                        kind,
                        range: node_range(&child),
                        selection_range: node_range(&name_node),
                        children: vec![],
                    });
                }
            }
        } else if child.kind() == "target_list" {
            collect_target_symbols(&child, src, kind, symbols);
        }
    }
}

fn collect_target_defs(node: tree_sitter::Node, src: &str, name: &str, defs: &mut Vec<Range>) {
    if node.kind() == "target" {
        if let Some(name_node) = node.child_by_field_name("name") {
            if node_text(&name_node, src) == name {
                defs.push(node_range(&name_node));
            }
        }
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            collect_target_defs(child, src, name, defs);
        }
    }
}

fn collect_refs(node: tree_sitter::Node, src: &str, name: &str, refs: &mut Vec<Range>) {
    // Match @ref references and identifiers
    if (node.kind() == "identifier" || node.kind() == "at_ref")
        && node_text(&node, src).trim_start_matches('@') == name
    {
        refs.push(node_range(&node));
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            collect_refs(child, src, name, refs);
        }
    }
}

static BUILTINS: [BuiltinInfo; 46] = [
    // ── Pipe functions ──
    BuiltinInfo {
        name: "to_json",
        signature: "to_json",
        documentation: "Convert value to JSON string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "to_str",
        signature: "to_str",
        documentation: "Convert value to string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "base64_encode",
        signature: "base64_encode",
        documentation: "Encode a string to Base64.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "base64_decode",
        signature: "base64_decode([encoding])",
        documentation: "Decode a Base64-encoded string. Optional encoding: Utf8/Gbk/Imap/...",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "html_escape",
        signature: "html_escape",
        documentation: "Escape HTML entities.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "html_unescape",
        signature: "html_unescape",
        documentation: "Unescape HTML entities.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "json_escape",
        signature: "json_escape",
        documentation: "Escape a string for JSON.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "json_unescape",
        signature: "json_unescape",
        documentation: "Unescape a JSON string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "str_escape",
        signature: "str_escape",
        documentation: "Escape special characters in a string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "url",
        signature: "url(domain|host|uri|path|params)",
        documentation: "Extract a URL component.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "path",
        signature: "path(name|path)",
        documentation: "Extract a file path component.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "nth",
        signature: "nth(index)",
        documentation: "Get the nth element from an array.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "get",
        signature: "get(field[/subfield])",
        documentation: "Get an object field value by name, supports nested paths.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "skip_empty",
        signature: "skip_empty",
        documentation: "Skip the value if it is empty.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "ip4_to_int",
        signature: "ip4_to_int",
        documentation: "Convert an IPv4 address to an integer.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "starts_with",
        signature: "starts_with('prefix')",
        documentation: "Check if string starts with the given prefix. Usable in pipe and match.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "map_to",
        signature: "map_to(value)",
        documentation: "Map input to a constant value (string, number, or bool).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "extract_main_word",
        signature: "extract_main_word",
        documentation: "Extract the first non-empty word from the input.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "extract_subject_object",
        signature: "extract_subject_object",
        documentation: "Extract subject/action/object/status structure from log text.",
        kind: CompletionItemKind::FUNCTION,
    },
    // ── Time functions ──
    BuiltinInfo {
        name: "Now::time",
        signature: "Now::time() -> time",
        documentation: "Current timestamp.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Now::date",
        signature: "Now::date() -> chars",
        documentation: "Current date string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Now::hour",
        signature: "Now::hour() -> digit",
        documentation: "Current hour (0-23).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Time::to_ts",
        signature: "Time::to_ts",
        documentation: "Convert time to Unix timestamp in seconds (UTC+8).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Time::to_ts_ms",
        signature: "Time::to_ts_ms",
        documentation: "Convert time to Unix timestamp in milliseconds (UTC+8).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Time::to_ts_us",
        signature: "Time::to_ts_us",
        documentation: "Convert time to Unix timestamp in microseconds (UTC+8).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Time::to_ts_zone",
        signature: "Time::to_ts_zone(timezone, unit)",
        documentation: "Convert time to timestamp with specified timezone and unit (ms/us/ss/s).",
        kind: CompletionItemKind::FUNCTION,
    },
    // ── Match functions ──
    BuiltinInfo {
        name: "ends_with",
        signature: "ends_with('suffix')",
        documentation: "Check if string ends with the given suffix.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "contains",
        signature: "contains('substring')",
        documentation: "Check if string contains the given substring.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "regex_match",
        signature: "regex_match('pattern')",
        documentation: "Check if string matches the given regex pattern.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "iequals",
        signature: "iequals('value')",
        documentation: "Case-insensitive string equality check.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "is_empty",
        signature: "is_empty()",
        documentation: "Check if the value is empty.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gt",
        signature: "gt(number)",
        documentation: "Greater-than comparison for match conditions.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "lt",
        signature: "lt(number)",
        documentation: "Less-than comparison for match conditions.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "eq",
        signature: "eq(number)",
        documentation: "Equality comparison with float tolerance for match conditions.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "in_range",
        signature: "in_range(min, max)",
        documentation: "Check if value is within the closed range [min, max].",
        kind: CompletionItemKind::FUNCTION,
    },
    // ── Privacy types ──
    BuiltinInfo {
        name: "privacy_ip",
        signature: "privacy_ip",
        documentation: "Mask IP address.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_specify_ip",
        signature: "privacy_specify_ip",
        documentation: "Mask specified IP address fields.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_id_card",
        signature: "privacy_id_card",
        documentation: "Mask ID card number.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_mobile",
        signature: "privacy_mobile",
        documentation: "Mask mobile phone number.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_mail",
        signature: "privacy_mail",
        documentation: "Mask email address.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_domain",
        signature: "privacy_domain",
        documentation: "Mask domain name.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_specify_name",
        signature: "privacy_specify_name",
        documentation: "Mask specified name fields.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_specify_domain",
        signature: "privacy_specify_domain",
        documentation: "Mask specified domain fields.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_specify_address",
        signature: "privacy_specify_address",
        documentation: "Mask specified address fields.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_specify_company",
        signature: "privacy_specify_company",
        documentation: "Mask specified company name fields.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
    BuiltinInfo {
        name: "privacy_keymsg",
        signature: "privacy_keymsg",
        documentation: "Mask key message fields.",
        kind: CompletionItemKind::ENUM_MEMBER,
    },
];
