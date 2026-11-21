//! Removes machine-identifying paths from free text before it leaves the workspace.
//!
//! This is deliberately conservative: it over-redacts rather than under-redacts, and it only
//! touches text fields the application wrote itself (error messages, operator notes). Hashed
//! records are never rewritten, because that would break their hashes.

const WORKSPACE: &str = "<workspace>";
const USER: &str = "<user>";
/// Directory names that introduce a per-user path segment, matched case-insensitively.
const USER_MARKERS: [&str; 3] = ["\\users\\", "/users/", "/home/"];

#[derive(Debug, Clone)]
pub struct Redactor {
    /// Both separator styles of each known root, longest first.
    roots: Vec<String>,
}

impl Redactor {
    /// `roots` are absolute locations that must never appear, e.g. the workspace directory.
    pub fn new(roots: &[String]) -> Self {
        let mut all = Vec::new();
        for root in roots {
            let trimmed = root.trim_end_matches(['\\', '/']);
            if trimmed.is_empty() {
                continue;
            }
            for variant in [trimmed.replace('/', "\\"), trimmed.replace('\\', "/")] {
                if !all.contains(&variant) {
                    all.push(variant);
                }
            }
        }
        all.sort_by_key(|r| std::cmp::Reverse(r.len()));
        Self { roots: all }
    }

    pub fn redact(&self, text: &str) -> String {
        redact_users(&self.redact_roots(text))
    }

    fn redact_roots(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        'scan: while !rest.is_empty() {
            for root in &self.roots {
                if rest.starts_with(root.as_str()) {
                    out.push_str(WORKSPACE);
                    rest = &rest[root.len()..];
                    continue 'scan;
                }
            }
            let ch = rest.chars().next().expect("rest is not empty");
            out.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
        out
    }
}

fn redact_users(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let hit = USER_MARKERS
            .iter()
            .filter(|m| lower[at..].starts_with(**m))
            .map(|m| m.len())
            .next();
        match hit {
            Some(marker_len) => {
                out.push_str(&text[at..at + marker_len]);
                at += marker_len;
                let rest = &text[at..];
                let end = rest
                    .find(|c: char| {
                        c == '\\'
                            || c == '/'
                            || c == ':'
                            || c.is_whitespace()
                            || c == '"'
                            || c == '\''
                    })
                    .unwrap_or(rest.len());
                if end > 0 && !rest.starts_with('<') {
                    out.push_str(USER);
                } else {
                    out.push_str(&rest[..end]);
                }
                at += end;
            }
            None => {
                let ch = text[at..].chars().next().expect("at is inside the text");
                out.push(ch);
                at += ch.len_utf8();
            }
        }
    }
    out
}
