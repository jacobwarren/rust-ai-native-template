//! Separate offline tools sharing a strict content model and explicit feedback export.
use crate::{config::Result, discovery_feedback, history, task_artifact::Artifact};
pub const PAGES: [&str; 5] = [
    "blindspots.html",
    "interventions.html",
    "interview.html",
    "reference.html",
    "plan.html",
];
const TITLES: [&str; 5] = [
    "Blindspot pass",
    "Brainstorm the intervention",
    "The interview",
    "Point at a reference",
    "The tweakable plan",
];
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
pub fn render(a: &Artifact, page: &str) -> Result<String> {
    let d = a.discoveries.last().ok_or("missing discovery")?;
    let i = PAGES
        .iter()
        .position(|p| *p == page)
        .ok_or("unknown discovery page")?;
    let data = serde_json::json!({"task":a.task,"page":page,"content":d,
        "discovery_sha256":discovery_feedback::discovery_hash(a)?,
        "feedback_base_sha256":history::json_digest(&a.feedback)?,
        "responses":discovery_feedback::responses(a)?});
    let encoded = serde_json::to_string(&data)?
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e");
    let mut s = include_str!("task_view.html").replace(
        "Task fieldnotes — lifecycle review",
        &format!("{} · {}", esc(&a.task), TITLES[i]),
    );
    s.push_str("<style>");
    s.push_str(include_str!("discovery.css"));
    s.push_str("</style><nav aria-label=\"Discovery artifacts\"><a href=\"unknowns.html\">Discovery index</a>");
    for (j, p) in PAGES.iter().enumerate() {
        s.push_str(&format!(
            "<a href=\"{p}\"{}>{}</a>",
            if i == j { " aria-current=\"page\"" } else { "" },
            TITLES[j]
        ));
    }
    s.push_str(&format!("</nav><header><span class=eyebrow>BEFORE IMPLEMENTATION · {}</span><h1>{}</h1><p class=lede>{}</p></header><main><div id=workspace></div><noscript>This discovery tool needs JavaScript for local interaction. Read record.json for its source content.</noscript><aside class=decision-desk><span class=eyebrow>YOUR RESPONSE</span><h2>Turn exploration into decisions</h2><p>Selections stay in this browser until you export and explicitly import them. They are not approvals.</p><pre id=decision-summary></pre><label class=actor-label>Your name or role<input id=actor autocomplete=name placeholder=\"Required for export\"></label><div class=actions><button id=copy-summary type=button>Copy summary</button><button id=export-feedback type=button>Export decisions (.json)</button><button id=reset-draft type=button class=secondary>Discard draft changes</button></div><p id=export-status role=status></p><details><summary>Record these decisions in the harness</summary><code>cargo xtask task-artifact decide --task {} --input PATH-TO-EXPORTED-JSON</code><p>After import, reopen regenerated pages. Older open pages deliberately cannot overwrite newer feedback.</p></details></aside></main><footer><a href=\"index.html\">Task overview</a> · <a href=\"record.json\">Source and revision history</a><p>Recommendations are labeled. Exploration does not change source files or grant approval.</p></footer>",esc(&a.task),TITLES[i],esc(&d.scope),esc(&a.task)));
    s.push_str("<script type=application/json id=discovery-data>");
    s.push_str(&encoded);
    s.push_str("</script><script>");
    s.push_str(include_str!("discovery_state.js"));
    s.push_str("</script><script>");
    s.push_str(include_str!("discovery_ui.js"));
    s.push_str("</script></body></html>\n");
    Ok(s)
}
