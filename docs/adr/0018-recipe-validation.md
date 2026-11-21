# ADR 0018: Recipe validation is a new revision with physical evidence and a sign-off

Status: accepted, 2026-11-21

## Decision
- `RecipeStatus` gains `Validated`. A validated recipe carries a `RecipeValidation`: pressed-sample
  evidence and a named sign-off. Both are required; a recipe is `Validated` exactly when it carries
  one. The field is omitted from the record when absent, so recipes that predate validation keep
  their content hash.
- Validation creates a **new revision** of the same recipe id (`Workspace::validate_recipe`, CLI
  `recipe validate`). The revision it was made from is never changed and keeps its jobs. Every
  policy is copied unchanged; only the status, revision, time and evidence differ.
- The evidence records who pressed and measured, the tolerances the samples were measured against,
  and each sample (session, label, within tolerance, note). The product requirements call a recipe
  validated only when repeated pressed samples meet agreed tolerances across sessions, so the domain
  requires at least 3 samples from at least 2 sessions, every one within tolerance. Those two
  numbers are the floor for "repeated" and "across sessions", not a tolerance: the tolerances are the
  operator's own words and are not interpreted.
- The evidence names the revision it validates, the setup and the mode. The setup must be physically
  verified and the mode validated, and the mode must belong to the setup. `import_recipe` applies the
  same conditions, so a record cannot claim validation under conditions this workspace has not
  established.
- The submission gate now requires a validated recipe whose evidence names this job's setup and mode.
  An experimental recipe, a retired one, or one validated under a different setup or mode blocks
  submission and touches nothing.
- Changing a validated recipe's white settings (ADR 0015) produces an experimental revision with no
  evidence. What validated the old recipe does not carry to a different one.

## Honest limits
- The evidence is attested by the operator. Nothing measures a pressed sample or reads a
  spectrophotometer; the system records the claim, who made it, and what it was bound to.
- No physical submission path exists yet, so nothing in this workspace can produce the samples. The
  evidence comes from prints made elsewhere (for example the commercial RIP workflow) and is entered
  by hand; this does not make any printer behaviour a verified claim.
- Validation is not re-checked when a validated recipe is imported inside an archive, which carries
  the record as written.
