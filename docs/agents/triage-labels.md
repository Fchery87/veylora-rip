# Triage Labels

This repository uses the default triage vocabulary:

| Role | Label |
| --- | --- |
| Needs triage | `needs-triage` |
| Needs information | `needs-info` |
| Ready for agent | `ready-for-agent` |
| Ready for human | `ready-for-human` |
| Won't fix | `wontfix` |

These labels are metadata for local markdown issues. They do not require a hosted issue tracker.

## Usage

- New or unclassified work: `needs-triage`
- Blocked by missing requirements or evidence: `needs-info`
- Clearly scoped and ready for implementation: `ready-for-agent`
- Requires user or operator review: `ready-for-human`
- Explicitly declined or outside scope: `wontfix`

A local issue may record labels in front matter or a plain `Labels` section.
