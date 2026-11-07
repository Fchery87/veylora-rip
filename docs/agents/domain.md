# Domain Documentation Rules

## Layout

This is a single-context project.

- Domain glossary: `GLOSSARY.md`
- Architecture decision records: `docs/adr/`

There is no monorepo-specific glossary map.

## Consumer rules

Before making architectural or domain changes:

1. Read `GLOSSARY.md`.
2. Search `docs/adr/` for decisions related to the change.
3. Preserve established terminology unless an ADR explicitly changes it.
4. Add or update an ADR when introducing a consequential architectural decision.
5. Update `GLOSSARY.md` when introducing a durable domain term or changing the meaning of an existing term.
6. Link implementation plans and issues to relevant glossary terms and ADRs where useful.

## ADR naming

Use:

```text
docs/adr/NNNN-short-decision-title.md
```

Each ADR should include:

- Status
- Date
- Context
- Decision
- Alternatives considered
- Consequences
- Validation or follow-up requirements

## Glossary naming

Use concise definitions for terms such as:

- Logical channel
- Physical slot
- Protocol channel
- PrintMode
- DeviceSetup
- Recipe
- Job snapshot
- Submission attempt
- Ambiguous submission
- Screened device raster
- White policy
