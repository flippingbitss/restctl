use std::{cell::LazyCell, ops::Deref, sync::LazyLock};

use egui::Align;
use tree_sitter::{Node, Point};

use crate::code::builder::AutoCompletionItem;

#[derive(Debug)]
enum JsonPathEntry<'a> {
    StringKey(&'a str),
    Array,
}

#[derive(Debug, PartialEq, Eq)]
enum CompletionQueryKind {
    ForKeys,
    ForValues,
}

#[derive(Debug)]
pub struct CompletionQuery<'a> {
    kind: CompletionQueryKind,
    path: Vec<JsonPathEntry<'a>>,
    partial_input: Option<&'a str>,
}

#[derive(Debug)]
struct JsonShapeGraphNode {
    name: &'static str,
    children: Vec<JsonShapeGraphNode>,
    values: Vec<&'static str>,
}

impl JsonShapeGraphNode {
    fn leaf(name: &'static str) -> Self {
        Self {
            name,
            children: Vec::new(),
            values: Vec::new(),
        }
    }

    fn with_values(name: &'static str, values: Vec<&'static str>) -> Self {
        Self {
            name,
            values,
            children: Vec::new(),
        }
    }

    fn with_children(name: &'static str, children: Vec<JsonShapeGraphNode>) -> Self {
        Self {
            name,
            children,
            values: Vec::new(),
        }
    }
}

#[rustfmt::skip]
static DUMMY_JSON_SHAPE: LazyLock<JsonShapeGraphNode> = LazyLock::new(|| {
    JsonShapeGraphNode::with_children("root", vec![
        JsonShapeGraphNode::with_children("user", vec![
            JsonShapeGraphNode::with_values("id", vec!["a", "b", "c"]),
            JsonShapeGraphNode::leaf("name"),
            JsonShapeGraphNode::leaf("email"),
            JsonShapeGraphNode::with_children("address", vec![
                JsonShapeGraphNode::leaf("street"),
                JsonShapeGraphNode::with_values("city", vec!["Toronto", "Vancouver", "Vaughan", "Brampton"]),
                JsonShapeGraphNode::leaf("zip"),
            ]),
        ]),
        JsonShapeGraphNode::with_children("settings", vec![
            JsonShapeGraphNode::with_values("theme", vec!["Dark", "Light", "System"]),
            JsonShapeGraphNode::leaf("notifications"),
        ])
    ])
});

pub fn get_autocompletion_tokens(query: CompletionQuery) -> Vec<AutoCompletionItem> {
    let mut current = DUMMY_JSON_SHAPE.deref();
    let mut active_node = Some(current);
    for entry in query.path.iter() {
        for node in current.children.iter() {
            match entry {
                JsonPathEntry::StringKey(path_key) => {
                    if *path_key == node.name {
                        current = node;
                        active_node = Some(node);
                    }
                }
                JsonPathEntry::Array => unreachable!(),
            }
        }
    }
    match query.kind {
        CompletionQueryKind::ForKeys => active_node
            .map(|node| {
                node.children
                    .iter()
                    .map(|c| AutoCompletionItem::title(c.name.to_owned()))
                    .collect::<Vec<AutoCompletionItem>>()
            })
            .unwrap_or_default(),
        CompletionQueryKind::ForValues => active_node
            .map(|n| {
                n.values
                    .iter()
                    .map(|&v| AutoCompletionItem::title(v.to_owned()))
                    .collect::<Vec<AutoCompletionItem>>()
            })
            .unwrap_or_default(),
    }
}

pub fn autocomplete_at_cursor<'a>(
    text: &'a str,
    root: tree_sitter::Node<'a>,
    cursor_byte_offset: usize,
    cursor_point: tree_sitter::Point,
    autocompletion_fn: impl FnOnce(CompletionQuery<'a>) -> Vec<AutoCompletionItem>,
) -> (String, Vec<AutoCompletionItem>) {
    let query =
        build_completion_query_for_active_string(text, root, cursor_byte_offset, cursor_point);
    let mut info_str = String::new();
    info_str.push_str(&format!("Query: {:?}\n", query));

    let completions = query
        .map(|value| {
            let partial_input = value.partial_input;
            autocompletion_fn(value)
                .into_iter()
                .filter(|x| {
                    let is_match = partial_input
                        .map(|p| x.title.starts_with(p) && x.title.len() > p.len()) // if only a partial match
                        .unwrap_or(true);
                    is_match
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // TODO: get rid the String and return modelled completion items
    info_str.push_str(&format!(
        "Completions: {:?}",
        completions
            .iter()
            .map(|x| x.title.as_str())
            .collect::<Vec<_>>()
    ));

    (info_str, completions)
}

fn get_pair_as_key_value<'a>(node: Option<Node<'a>>) -> (Option<Node<'a>>, Option<Node<'a>>) {
    node.map(|node| {
        (
            node.child_by_field_name("key"),
            node.child_by_field_name("value"),
        )
    })
    .unwrap_or_default()
}

fn get_ancestor_pair_node<'a>(node: Node<'a>) -> Option<Node<'a>> {
    if node.kind() == "pair" {
        return Some(node);
    }
    let mut current_node = node.parent();
    while let Some(current) = current_node {
        if current.kind() == "pair" {
            return Some(current);
        }
        current_node = current.parent();
    }
    None
}

fn cursor_inside_node<'a>(node: Node<'a>, cursor_byte_offset: usize) -> bool {
    node.byte_range().contains(&cursor_byte_offset)
}

// We are only handling strings for now,
fn build_completion_query_for_active_string<'a>(
    text_buffer: &'a str,
    root: tree_sitter::Node<'a>,
    cursor_byte_offset: usize,
    cursor_point: Point,
) -> Option<CompletionQuery<'a>> {
    let point_to_left = Point::new(cursor_point.row, cursor_point.column.saturating_sub(1));
    let active_node = root.named_descendant_for_point_range(cursor_point, cursor_point);
    if let Some(node) = active_node {
        // we are not editing a string, so exit
        if !(node.kind() == "string" || node.kind() == "string_content") {
            return None;
        }

        let prefix = match node.kind() {
            "string" => node.named_child(0).map(|n| &text_buffer[n.byte_range()]),
            "string_content" => Some(&text_buffer[node.byte_range()]),
            _ => None,
        };

        let pair_node = get_ancestor_pair_node(node);
        let (key_node, value_node) = get_pair_as_key_value(pair_node);

        let mut completion_kind = None;

        // Case 1 for Keys (with valid value after colon)
        // {"|": ".."}
        if let Some(key) = key_node {
            if cursor_inside_node(key, cursor_byte_offset) {
                // show key completions
                completion_kind = Some(CompletionQueryKind::ForKeys);
            }
        }

        // Case 2 for values:
        // {"abc": "|"}
        // {"abc": "de|"}
        if let Some(value) = value_node {
            if cursor_inside_node(value, cursor_byte_offset) {
                // show value completions
                completion_kind = Some(CompletionQueryKind::ForValues);
            }
        }

        // Case 3: Dangling Keys (no color or value)
        // {"|"}
        // {"|":}
        // {"abc": {"|"}}
        //
        // this is actually the most common case, when user is trying
        // to explore the json graph
        let is_dangling_key = match node.kind() {
            "string" => node.parent().is_some_and(|n| n.is_error()),
            "string_content" => node
                .parent()
                .and_then(|node| node.parent())
                .is_some_and(|n| n.is_error()),
            _ => false,
        };
        if is_dangling_key {
            completion_kind = Some(CompletionQueryKind::ForKeys);
        }
        //
        // log::debug!(
        //     "current node: {}, error = {}, missing = {}",
        //     node.kind(),
        //     node.is_error(),
        //     node.is_missing()
        // );
        //
        if let Some(query_kind) = completion_kind {
            let mut path = Vec::new();
            let mut current_node = pair_node;
            while let Some(current) = current_node {
                if let Some(key_str_node) = current.child_by_field_name("key") {
                    if let Some(key_str_content_node) = key_str_node.named_child(0) {
                        let key_contents = &text_buffer[key_str_content_node.byte_range()];
                        let entry = JsonPathEntry::StringKey(key_contents);
                        path.push(entry);
                    } else {
                        path.push(JsonPathEntry::StringKey(""));
                    }
                }
                current_node = current.parent();
            }
            path.reverse();
            if query_kind == CompletionQueryKind::ForKeys && !is_dangling_key {
                path.pop();
            }

            return Some(CompletionQuery {
                path,
                kind: query_kind,
                partial_input: prefix,
            });
        }
    }

    None
}
