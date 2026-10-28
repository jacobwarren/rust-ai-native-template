//! Fixed offline presentation. Task-authored text is always escaped, never executed.
use crate::task_artifact::{After, Artifact};
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn block(label: &str, text: &str) -> String {
    format!(
        "<div class=block><h3>{}</h3><p>{}</p></div>",
        escape(label),
        escape(text)
    )
}
fn refs(values: &[String]) -> String {
    let mut s = String::from("<ul class=refs>");
    for r in values {
        let href = if r.starts_with("https://") {
            r.clone()
        } else {
            // Encode local filenames so '#' and '?' cannot redirect the reference.
            let encoded: String = r
                .bytes()
                .map(|b| {
                    if b.is_ascii_alphanumeric() || b"/-_.".contains(&b) {
                        char::from(b).to_string()
                    } else {
                        format!("%{b:02X}")
                    }
                })
                .collect();
            format!("../../../{encoded}")
        };
        s.push_str(&format!(
            "<li><a href=\"{}\" rel=\"noreferrer\">{}</a></li>",
            escape(&href),
            escape(r)
        ));
    }
    s.push_str("</ul>");
    s
}
fn brief(c: &After) -> String {
    [
        block("What changed and why", &c.change),
        block("Demonstration / before and after", &c.demonstration),
        block("Verification and boundaries", &c.verification),
        block("Risks, tradeoffs and decisions needed", &c.risks),
        refs(&c.references),
    ]
    .concat()
}
pub const PAGES: [&str; 5] = [
    "index.html",
    "unknowns.html",
    "implementation.html",
    "buy-in.html",
    "quiz.html",
];
const TITLES: [&str; 5] = [
    "Task review",
    "Unknowns before implementation",
    "Implementation notes",
    "The case for this change",
    "Quiz before you merge",
];
fn pending(s: &str) -> String {
    format!("<p class=pending>{}</p>", escape(s))
}
fn before(a: &Artifact) -> String {
    let mut s = String::new();
    if !a.discoveries.is_empty() {
        s.push_str("<p class=intro>Five separate tools for investigating uncertainty and recording decisions. Open each artifact to explore, compare, and respond.</p><div class=artifact-grid>");
        for (page, title, description) in [
            (
                "blindspots.html",
                "Blindspot pass",
                "Evidence, consequences and constraints assembled into a brief.",
            ),
            (
                "interventions.html",
                "Brainstorm the intervention",
                "Compare approaches across effort and select what resonates.",
            ),
            (
                "interview.html",
                "The interview",
                "Answer consequential questions, revisit decisions and export a specification.",
            ),
            (
                "reference.html",
                "Point at a reference",
                "Inspect matched semantics and correct the interpretation.",
            ),
            (
                "plan.html",
                "The tweakable plan",
                "Explore alternatives with live interface previews before execution.",
            ),
        ] {
            s.push_str(&format!("<a class=artifact-card href=\"{page}\"><h2>{title}</h2><p>{description}</p><span class=open>Open artifact →</span></a>"));
        }
        s.push_str("</div><details><summary>Original preflight snapshot</summary>");
    }
    for (k, v) in [
        ("Unknowns", &a.before.unknowns),
        ("Assumptions", &a.before.assumptions),
        ("Alternatives and choices", &a.before.alternatives),
        ("Expected outcome and verification", &a.before.verification),
    ] {
        s.push_str(&block(k, v));
    }
    s.push_str(&refs(&a.before.references));
    if !a.discoveries.is_empty() {
        s.push_str("</details>");
    }
    s
}
fn notes(a: &Artifact) -> String {
    let mut s = String::new();
    if a.notes.is_empty() {
        return pending("Implementation has not been recorded yet.");
    }
    for (i, n) in a.notes.iter().enumerate() {
        s.push_str(&format!(
            "<article class=note><span class=eyebrow>NOTE {:02}</span>{}{}{}{}</article>",
            i + 1,
            block("Observation / changed assumption", &n.observation),
            block("Decision and rationale", &n.decision),
            block("Effect on behavior and next steps", &n.effect),
            refs(&n.references)
        ));
    }
    s
}
fn publications(a: &Artifact) -> String {
    let Some(p) = a.publications.last() else {
        return pending("Buy-in brief pending. No finished behavior is claimed by this page.");
    };
    let mut s = String::new();
    if !crate::task_artifact::publication_current(a) {
        s.push_str(&pending(
            "Implementation notes, discovery or decisions changed. This brief needs an updated publication.",
        ));
    }
    s.push_str(&brief(&p.content));
    for (i, old) in a
        .publications
        .iter()
        .enumerate()
        .take(a.publications.len() - 1)
    {
        s.push_str(&format!(
            "<details class=revision><summary>Previous brief {}</summary>{}</details>",
            i + 1,
            brief(&old.content)
        ));
    }
    s
}
fn quiz(a: &Artifact) -> String {
    let mut s = String::from(
        "<p class=intro>Explore the scenarios, then check your answers. This is a learning aid, not approval to merge.</p>",
    );
    let Some(p) = a.publications.last() else {
        s.push_str(&pending(
            "Scenario quiz pending publication of the reviewer brief.",
        ));
        return s;
    };
    if !crate::task_artifact::publication_current(a) {
        s.push_str(&pending(
            "This quiz predates the latest notes, discovery or decisions. An updated publication is needed.",
        ));
    }
    for (i, q) in p.content.quiz.iter().enumerate() {
        s.push_str(&format!("<article class=question data-answer=\"{}\"><span class=eyebrow>QUESTION {:02}</span><h2>{}</h2><fieldset><legend class=sr-only>{}</legend>",q.answer,i+1,escape(&q.question),escape(&q.question)));
        for (j, o) in q.options.iter().enumerate() {
            s.push_str(&format!(
                "<label><input type=radio name=q{i} value={j}><span>{}</span></label>",
                escape(o)
            ));
        }
        s.push_str(&format!("</fieldset><button type=button class=check>Check answer</button><p class=feedback aria-live=polite></p><details class=explanation><summary>Explanation and evidence</summary><p>{}</p>{}</details></article>",escape(&q.explanation),refs(&q.references)));
    }
    s
}
fn index(a: &Artifact) -> String {
    let states = [
        "Preserved initial snapshot",
        if a.notes.is_empty() {
            "Pending"
        } else {
            "Notes recorded"
        },
        if crate::task_artifact::publication_current(a) {
            "Published"
        } else {
            "Pending / needs publication"
        },
        if crate::task_artifact::publication_current(a) {
            "Ready for self-check"
        } else {
            "Pending / needs publication"
        },
    ];
    let descriptions = [
        "Explore assumptions, alternatives, and what needs to be learned before building.",
        "Follow discoveries, decisions, and changes of direction as the work progresses.",
        "Understand the behavior, demonstration, evidence, and tradeoffs behind the change.",
        "Test your understanding with scenario questions and evidence-backed explanations.",
    ];
    let mut s = String::from("<div class=artifact-grid>");
    for i in 1..5 {
        s.push_str(&format!("<a class=artifact-card href=\"{}\"><span class=eyebrow>ARTIFACT 0{i} · {}</span><h2>{}</h2><p>{}</p><span class=open>Open artifact →</span></a>", PAGES[i],states[i-1],TITLES[i],descriptions[i-1]));
    }
    s.push_str("</div>");
    s
}
pub fn render(a: &Artifact, page: &str) -> String {
    let position = PAGES.iter().position(|p| *p == page).unwrap_or(0);
    let mut s = include_str!("task_view.html").replace(
        "Task fieldnotes — lifecycle review",
        &format!("{} · {}", escape(&a.task), TITLES[position]),
    );
    s.push_str("<nav aria-label=\"Task artifacts\">");
    for (i, name) in [
        "Overview",
        "01 Unknowns",
        "02 Implementation",
        "03 Buy-in",
        "04 Quiz",
    ]
    .iter()
    .enumerate()
    {
        s.push_str(&format!(
            "<a href=\"{}\"{}>{name}</a>",
            PAGES[i],
            if i == position {
                " aria-current=\"page\""
            } else {
                ""
            }
        ));
    }
    s.push_str(&format!("</nav><header><span class=eyebrow>TASK FIELDNOTES · {}</span><h1>{}</h1><p class=lede>{}</p></header><main>",escape(&a.task),TITLES[position],escape(&a.before.outcome)));
    s.push_str(&match position {
        1 => before(a),
        2 => notes(a),
        3 => publications(a),
        4 => quiz(a),
        _ => index(a),
    });
    s.push_str(&format!("</main><footer><a href=\"index.html\">All artifacts</a> · <a href=\"record.json\">Source record</a><details><summary>Provenance and verification boundary</summary><p class=identity>Initial revision: {}<br>Initial source fingerprint: {}</p><p>Preserved snapshots and appended revisions support review. They do not attest to chronology, correctness, or approval.</p></details></footer>",escape(a.source_revision.as_str().unwrap_or("unavailable")),escape(&a.source_sha256)));
    if position == 4 {
        s.push_str("<script>");
        s.push_str(include_str!("task_quiz.js"));
        s.push_str("</script>");
    }
    s.push_str("</body></html>\n");
    s
}
