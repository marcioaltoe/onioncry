use crate::GraphReport;
use std::collections::BTreeMap;

pub fn render_graph_mermaid(report: &GraphReport) -> String {
    let mut output = String::from("graph TD\n");
    let mut mermaid_ids = BTreeMap::<&str, String>::new();

    for (index, node) in report.nodes.iter().enumerate() {
        let mermaid_id = format!("n{index}");
        mermaid_ids.insert(&node.id, mermaid_id.clone());
        output.push_str(&format!(
            "  {mermaid_id}[\"{}\"]\n",
            escape_mermaid_label(&node.label)
        ));
    }

    for edge in &report.edges {
        let Some(from) = mermaid_ids.get(edge.from.as_str()) else {
            continue;
        };
        let Some(to) = mermaid_ids.get(edge.to.as_str()) else {
            continue;
        };
        let label = edge_label(edge.via.as_deref(), edge.import_count);
        if label.is_empty() {
            output.push_str(&format!("  {from} --> {to}\n"));
        } else {
            output.push_str(&format!(
                "  {from} -->|{}| {to}\n",
                escape_mermaid_label(&label)
            ));
        }
    }

    output
}

fn edge_label(via: Option<&str>, import_count: usize) -> String {
    match (via, import_count) {
        (Some(via), 0 | 1) => via.to_string(),
        (Some(via), count) => format!("{via} ({count})"),
        (None, 0 | 1) => String::new(),
        (None, count) => format!("{count} imports"),
    }
}

fn escape_mermaid_label(label: &str) -> String {
    label.replace('"', "\\\"").replace('|', "\\|")
}
