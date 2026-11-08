# Device setup boundary

This headless crate describes a configuration, not a printer driver. It performs no physical operations and provides no PrintMode, persistence, or native-support claim.

Public construction validates the channel map and physical inventory. A setup exposes immutable details. Changes go through `revise`, preserving the ID, incrementing the revision, recalculating the content hash, and discarding verification. The channel map has its own increasing version when bindings change.

Only complete labeled physical observations matching the setup ID, revision, content hash, map version, and every slot/protocol/logical assignment can establish channel verification. This is an operator evidence-recording boundary, not an authenticity detector: callers must supply genuine physical observations and retain their artifacts. Unit tests use synthetic observations only.

Production gating here verifies the setup alone. It never authorizes transport or replaces PrintMode, Recipe, backend, or physical transfer acceptance. Experimental reasons do not validate a setup. Imported/inferred details enter through ordinary construction and always start unverified. No deserialization API can inject a verified state.

Configuration hashing is version-tagged and length-delimited. It includes identity, revision, origin, every setup field, physical inventory, map version, and allocation fractions. Verification evidence is retained separately and bound to that immutable configuration hash to avoid a self-referential hash. Evidence changes do not rewrite configuration identity.
