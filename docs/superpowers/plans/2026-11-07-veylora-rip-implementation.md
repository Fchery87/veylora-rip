# Veylora RIP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Veylora RIP as a native Windows DTF RIP that can produce and submit independently controlled process and white output for one verified converted Epson ET-8550 configuration, then grow into a reliable personal production tool with future distribution-ready foundations.

**Architecture:** Use a Rust workspace with a deterministic, headless rendering core and thin adapters for persistence, Windows transport, CLI diagnostics, and a Tauri 2 desktop shell. The desktop frontend uses TypeScript, Svelte, Bits UI, and Vite+ as its unified Vite-based toolchain, with pnpm as the declared package manager. Separate logical artwork/separation planes from calibrated physical-slot rasters, screened device data, and printer encoding. Make the file sink and output inspector first-class so native protocol work can be tested without a UI. Gate every phase on artifacts and physical evidence rather than on feature count.

**Tech Stack:** Rust stable workspace; Tauri 2; TypeScript; Svelte; Bits UI; Vite+; pnpm; `image`/`png` or equivalent audited image decoders; Little CMS or a maintained Rust binding for ICC transforms; `serde` plus a versioned binary/JSON job manifest; SQLite or an equivalent local durable store; Windows spooler APIs through a narrow adapter; Cargo tests; Vite+ `vp check`, `vp test`, and `vp build`; Playwright for frontend behavior; property tests; golden fixtures; fault injection; and physical bench targets.

**Spec:** `docs/veylora-rip-prd.md`, `docs/native-rip-feature-requirements.md`, and `docs/native-rip-competitive-gap-analysis.md`.

## Global Constraints

- Native output without Digital Factory, AcroRIP, PrintFab, or another commercial RIP is the product success criterion.
- The first supported device is one converted Epson ET-8550 configuration, not the printer model in the abstract.
- Logical channels, physical slots, protocol channels, and screened drop codes are separate concepts.
- No production-ready mode may be created from unverified device facts.
- Every production job snapshots artwork, masks, transforms, device setup, print mode, recipe, profile hashes, software/backend versions, and output artifacts.
- Mirror, scaling, color conversion, white generation, calibration, screening, and encoding each have one owner.
- Unsupported input and unsupported device controls fail clearly before submission.
- Rendering is bounded, cancellable, and disk-space-aware from the first full-sheet implementation.
- Ambiguous submissions enter review and are never retried automatically.
- No automatic firmware updates, head cleaning, waste-counter changes, or ink servicing in the initial product.
- Commercial-ready foundations do not imply accounts, cloud queues, payments, subscriptions, telemetry, or DRM.
- Physical claims require tests on the actual converted printer, film, ink, press, cure, and garment process.

---

## Architecture and repository layout

Create the following workspace structure. Keep crates focused and make the dependency direction one-way.

```text
veylora-rip/
├── Cargo.toml
├── Cargo.lock
├── crates/
│   ├── veylora-domain/          # IDs, enums, immutable snapshots, validation errors
│   ├── veylora-image/           # PNG decoding, metadata, source assets, resource limits
│   ├── veylora-color/           # working spaces, ICC transforms, black/neutral policy
│   ├── veylora-layout/          # physical placement, transforms, gang-sheet model
│   ├── veylora-separation/      # CMYK, white policies, masks, choke/spread, clear mask
│   ├── veylora-screen/          # calibration application, ink limits, deterministic screens
│   ├── veylora-render/          # band renderer, intermediate artifacts, cancellation
│   ├── veylora-device/          # DeviceSetup, PrintMode, capability model, device registry
│   ├── veylora-encoder/         # ET-8550 protocol encoder and file sink
│   ├── veylora-transport/       # Windows spooler adapter and fake transport
│   ├── veylora-persistence/     # job store, attempts, recipe store, migrations
│   ├── veylora-diagnostics/     # target generation, reports, support bundles
│   ├── veylora-cli/             # headless commands for probe, render, inspect, submit
│   └── veylora-app/             # Tauri 2 application-service bridge
├── app/                         # Vite+ + Svelte + TypeScript frontend, using pnpm
├── fixtures/
│   ├── images/
│   ├── profiles/
│   ├── targets/
│   ├── expected/
│   └── physical/
├── device-config/
│   └── et8550/
├── docs/
│   └── bench/
├── scripts/
└── installer/
```

The directory name shown above is conceptual. Use the repository's actual path and avoid spaces in the on-disk package/workspace name. The product display name remains **Veylora RIP**.

Dependency rule:

```text
veylora-domain
  -> image, color, layout, separation, screen
  -> render
  -> device, encoder, persistence, diagnostics
  -> transport and CLI
  -> app UI
```

`veylora-domain` must not depend on Windows, a UI toolkit, a database, or a specific image decoder. Device encoding must consume already validated physical-slot raster bands and must not perform hidden color management or white generation.

---

## Phase 0: workspace and engineering foundation

### Task 0.1: Create the Rust workspace and CI baseline

**Files:**
- Create: `Cargo.toml`
- Create: `crates/veylora-domain/Cargo.toml`
- Create: `crates/veylora-domain/src/lib.rs`
- Create: `crates/veylora-cli/Cargo.toml`
- Create: `crates/veylora-cli/src/main.rs`
- Create: `.github/workflows/ci.yml`
- Create: `rustfmt.toml`
- Create: `clippy.toml`
- Test: `crates/veylora-domain/src/lib.rs`

**Interfaces:**
- Produces a compiling workspace and a CLI that reports its version.
- The workspace must expose `veylora-domain` without platform dependencies.

- [ ] **Step 1: Write the failing workspace smoke test**

Add a domain test that asserts the product version and a CLI test that invokes the binary with `--version` through the chosen command harness.

- [ ] **Step 2: Run the failing test**

Run: `cargo test --workspace`
Expected: FAIL because the workspace and version API do not exist.

- [ ] **Step 3: Implement the minimal workspace**

Define workspace members, package metadata, a `PRODUCT_NAME` constant, and a CLI `--version` command. Pin dependency versions in the lockfile and configure format/check/clippy jobs.

- [ ] **Step 4: Run the baseline checks**

Run: `cargo fmt --all -- --check`  
Run: `cargo clippy --workspace --all-targets -- -D warnings`  
Run: `cargo test --workspace`
Expected: all pass.

- [ ] **Step 5: Commit**

```text
git add Cargo.toml Cargo.lock crates .github rustfmt.toml clippy.toml
git commit -m "chore: create Veylora RIP Rust workspace"
```

### Task 0.2: Establish error, ID, hashing, and provenance primitives

**Files:**
- Create: `crates/veylora-domain/src/ids.rs`
- Create: `crates/veylora-domain/src/errors.rs`
- Create: `crates/veylora-domain/src/provenance.rs`
- Create: `crates/veylora-domain/tests/provenance.rs`

**Interfaces:**

```rust
pub struct JobId(Uuid);
pub struct AssetId(Uuid);
pub struct DeviceSetupId(Uuid);
pub struct PrintModeId(Uuid);
pub struct RecipeId(Uuid);
pub struct AttemptId(Uuid);
pub struct ContentHash([u8; 32]);
pub struct Provenance { pub software_version: String, pub backend_version: String, pub content_hashes: Vec<(String, ContentHash)> }
```

- [ ] **Step 1: Write tests for stable IDs and content hashes**
- [ ] **Step 2: Run `cargo test -p veylora-domain` and verify failure**
- [ ] **Step 3: Implement newtype IDs, SHA-256 hashing, and serializable provenance**
- [ ] **Step 4: Test round trips, unequal IDs, and hash changes**
- [ ] **Step 5: Commit**

```text
git add crates/veylora-domain
git commit -m "feat: add domain identifiers and provenance"
```

### Task 0.3: Define domain error taxonomy and validation severity

**Files:**
- Modify: `crates/veylora-domain/src/errors.rs`
- Create: `crates/veylora-domain/src/preflight.rs`
- Create: `crates/veylora-domain/tests/errors.rs`

**Interfaces:**

```rust
pub enum Severity { Blocking, ConfirmationRequired, Informational }
pub struct Finding { pub code: String, pub severity: Severity, pub message: String, pub subject: Option<String> }
pub trait Validate { fn validate(&self) -> Vec<Finding>; }
```

- [ ] **Step 1: Add tests for blocking versus informational findings**
- [ ] **Step 2: Verify they fail before implementation**
- [ ] **Step 3: Implement typed error categories and serializable findings**
- [ ] **Step 4: Verify stable error codes and human messages**
- [ ] **Step 5: Commit**

---

## Phase 1: device truth before application polish

### Task 1.1: Create the bench record and device-setup data model

**Files:**
- Create: `crates/veylora-device/src/setup.rs`
- Create: `crates/veylora-device/src/channels.rs`
- Create: `crates/veylora-device/src/lib.rs`
- Create: `crates/veylora-device/tests/setup_validation.rs`
- Create: `docs/bench/device-record-template.md`

**Interfaces:**

```rust
pub struct DeviceSetup { pub id: DeviceSetupId, pub identity: DeviceIdentity, pub firmware: Option<String>, pub physical_slots: Vec<PhysicalSlot>, pub channel_map: LogicalChannelMap, pub verification: VerificationState, pub content_hash: ContentHash }
pub struct LogicalInk { pub name: String }
pub struct PhysicalSlot { pub protocol_channel: u8, pub label: String, pub ink: Option<LogicalInk>, pub allocation: AllocationPolicy }
pub enum VerificationState { Unverified, Experimental, Verified { evidence_hash: ContentHash } }
```

- [ ] **Step 1: Test that unverified setups cannot be used for production**
- [ ] **Step 2: Run the device tests and verify failure**
- [ ] **Step 3: Implement setup validation and versioned channel mapping**
- [ ] **Step 4: Test duplicate-white allocation rules and content hashing**
- [ ] **Step 5: Write the bench record template for model, firmware, tank labels, ink, film, feed, reference mode, and observations**
- [ ] **Step 6: Commit**

### Task 1.2: Build the diagnostic target generator

**Files:**
- Create: `crates/veylora-diagnostics/src/targets.rs`
- Create: `crates/veylora-diagnostics/src/lib.rs`
- Create: `crates/veylora-diagnostics/tests/target_geometry.rs`
- Create: `fixtures/targets/channel-identity.json`

**Interfaces:**

```rust
pub enum TargetKind { ChannelIdentity, Geometry, Registration, WhiteRamp, WetLoad, AlphaEdge, Screening }
pub struct DiagnosticTarget { pub kind: TargetKind, pub width_mm: f64, pub height_mm: f64, pub objects: Vec<TargetObject> }
pub fn generate_target(kind: TargetKind, size: PhysicalSize) -> DiagnosticTarget;
```

- [ ] **Step 1: Test asymmetric text/marker, isolated channels, ruler, and overlap objects**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement deterministic target geometry in physical units**
- [ ] **Step 4: Test target serialization and stable hashes**
- [ ] **Step 5: Commit**

### Task 1.3: Write the native-protocol investigation CLI

**Files:**
- Create: `crates/veylora-device/src/probe.rs`
- Modify: `crates/veylora-cli/src/main.rs`
- Create: `crates/veylora-cli/tests/probe_help.rs`
- Create: `docs/bench/native-protocol-notes.md`

**Interfaces:**

```rust
pub struct ProbeObservation { pub command_family: String, pub framing: Option<String>, pub settings: Vec<ObservedSetting>, pub evidence_files: Vec<ContentHash> }
pub enum ProbeAction { InspectReference, RenderBoundedTarget, DecodeCandidate, RecordObservation }
```

- [ ] **Step 1: Add CLI command tests for `veylora probe --help` and file-sink-only operation**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement probe command that never submits without an explicit experimental flag**
- [ ] **Step 4: Record reference settings and compare one-variable-at-a-time observations**
- [ ] **Step 5: Add legal/provenance notes for captured or exported artifacts**
- [ ] **Step 6: Commit**

### Gate A: native device proof

Pause implementation after this gate and print the bounded target on the actual setup. Do not begin production UI work until the following evidence exists:

- physical channel identity for required CMY/K/W behavior;
- a candidate native stream generated by Veylora;
- correct layer order, orientation, scale, feed, and ejection;
- repeated physical output after restart and on fresh film;
- documented failure behavior for partial send and cancellation.

If the command path cannot be established, record the blocker and keep later phases disabled. Do not declare a prepress-only workflow to be native RIP success.

---

## Phase 2: immutable job and device configuration model

### Task 2.1: Implement versioned PrintMode and capability model

**Files:**
- Create: `crates/veylora-device/src/mode.rs`
- Create: `crates/veylora-device/src/capability.rs`
- Create: `device-config/et8550/mode-experimental.json`
- Create: `crates/veylora-device/tests/mode_compatibility.rs`

**Interfaces:**

```rust
pub struct PrintMode { pub id: PrintModeId, pub device_setup: DeviceSetupId, pub raster: RasterSpec, pub protocol: ProtocolSpec, pub media: MediaSpec, pub deposition: DepositionSpec, pub quality: QualitySpec, pub status: ModeStatus }
pub struct DeviceCapability { pub independent_white: Capability, pub process_channels: Vec<String>, pub mirror_owner: MirrorOwner, pub supported_controls: HashSet<ControlId> }
pub enum ModeStatus { Experimental, Validated, Retired }
pub fn check_compatibility(mode: &PrintMode, setup: &DeviceSetup, recipe: &PrintRecipe) -> Vec<Finding>;
```

- [ ] **Step 1: Test incompatible firmware, map, media, and profile combinations**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement immutable mode records and capability-driven controls**
- [ ] **Step 4: Add an experimental ET-8550 mode record containing only measured facts**
- [ ] **Step 5: Verify mode hashes change when any protocol or media field changes**
- [ ] **Step 6: Commit**

### Task 2.2: Implement portable job manifest and snapshot model

**Files:**
- Create: `crates/veylora-domain/src/job.rs`
- Create: `crates/veylora-domain/src/recipe.rs`
- Create: `crates/veylora-domain/tests/job_snapshot.rs`

**Interfaces:**

```rust
pub struct JobSnapshot { pub job_id: JobId, pub assets: Vec<AssetRef>, pub layout: LayoutSnapshot, pub setup: DeviceSetupId, pub mode: PrintModeId, pub recipe: RecipeId, pub provenance: Provenance, pub created_at: DateTime<Utc> }
pub struct PrintRecipe { pub id: RecipeId, pub calibration: CalibrationRef, pub profile: Option<ProfileRef>, pub white_policy: WhitePolicy, pub screen: ScreenSpec, pub finish: FinishConditions, pub status: RecipeStatus }
```

- [ ] **Step 1: Test snapshot immutability and hash stability**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement versioned serializable manifests**
- [ ] **Step 4: Reject mutation of a snapshot after render begins**
- [ ] **Step 5: Commit**

### Task 2.3: Implement local persistence and schema migrations

**Files:**
- Create: `crates/veylora-persistence/Cargo.toml`
- Create: `crates/veylora-persistence/src/lib.rs`
- Create: `crates/veylora-persistence/src/migrations.rs`
- Create: `crates/veylora-persistence/src/jobs.rs`
- Create: `crates/veylora-persistence/tests/migrations.rs`

**Interfaces:**

```rust
pub trait JobStore { fn create_snapshot(&self, job: JobSnapshot) -> Result<()>; fn get(&self, id: JobId) -> Result<JobSnapshot>; fn append_attempt(&self, attempt: SubmissionAttempt) -> Result<()>; }
pub trait Migration { fn version(&self) -> u32; fn apply(&self, db: &Connection) -> Result<()>; }
```

- [ ] **Step 1: Test create/read, append-only attempts, restart recovery, and migration from schema version 0**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement SQLite schema with immutable snapshots and append-only attempts**
- [ ] **Step 4: Add disk-full and corrupt-database errors**
- [ ] **Step 5: Commit**

---

## Phase 3: source import, color, and layout core

### Task 3.1: Implement bounded PNG import and metadata contract

**Files:**
- Create: `crates/veylora-image/src/png.rs`
- Create: `crates/veylora-image/src/limits.rs`
- Create: `crates/veylora-image/src/asset.rs`
- Create: `crates/veylora-image/tests/png_fixtures.rs`
- Add: `fixtures/images/hidden-rgb.png`
- Add: `fixtures/images/alpha-ramp.png`
- Add: `fixtures/images/unsupported-apng.png`
- Add: `fixtures/images/conflicting-color-tags.png`

**Interfaces:**

```rust
pub struct DecodeLimits { pub max_pixels: u64, pub max_profile_bytes: u64, pub max_dimension: u32 }
pub struct SourceAsset { pub id: AssetId, pub original_hash: ContentHash, pub metadata: ImageMetadata, pub profile: Option<ProfileBlob>, pub alpha: AlphaDescription }
pub fn decode_png(bytes: &[u8], limits: DecodeLimits) -> Result<SourceAsset, ImageError>;
```

- [ ] **Step 1: Add fixtures and tests for alpha, hidden RGB, profiles, conflicting tags, HDR, APNG, malformed files, and non-square PPI**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement bounded decoding and metadata precedence**
- [ ] **Step 4: Reject unsupported inputs with stable error codes**
- [ ] **Step 5: Commit**

### Task 3.2: Define color-space and ICC transform boundary

**Files:**
- Create: `crates/veylora-color/src/space.rs`
- Create: `crates/veylora-color/src/transform.rs`
- Create: `crates/veylora-color/src/lib.rs`
- Create: `crates/veylora-color/tests/transform_fixtures.rs`
- Add: `fixtures/profiles/srgb.icc`
- Add: `fixtures/profiles/test-cmyk.icc`

**Interfaces:**

```rust
pub struct ColorPolicy { pub rendering_intent: RenderingIntent, pub black_point_compensation: bool, pub untagged_rgb: UntaggedRgbPolicy }
pub struct ColorTransform { pub source: ColorSpace, pub working: WorkingSpace, pub policy: ColorPolicy, pub profile_hashes: Vec<ContentHash> }
pub fn transform_pixel(transform: &ColorTransform, value: SourcePixel) -> Result<WorkingPixel, ColorError>;
```

- [ ] **Step 1: Test known RGB values, untagged policy, profile hash recording, and no-double-transform behavior**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement the selected licensed ICC engine behind a narrow adapter**
- [ ] **Step 4: Test transform determinism and explicit unsupported profile errors**
- [ ] **Step 5: Commit**

### Task 3.3: Implement physical layout and transform ownership

**Files:**
- Create: `crates/veylora-layout/src/geometry.rs`
- Create: `crates/veylora-layout/src/placement.rs`
- Create: `crates/veylora-layout/src/transforms.rs`
- Create: `crates/veylora-layout/tests/orientation.rs`

**Interfaces:**

```rust
pub struct Placement { pub asset: AssetId, pub x_mm: f64, pub y_mm: f64, pub width_mm: f64, pub height_mm: f64, pub rotation: Rotation, pub mirror: Mirror }
pub struct PrintableBox { pub width_mm: f64, pub height_mm: f64, pub margins_mm: Margins }
pub fn resolve_transform(layout: &Layout, mode: &PrintMode) -> Result<ResolvedTransform, LayoutError>;
```

- [ ] **Step 1: Test ruler, asymmetric text, off-center marker, rotation, and one-mirror ownership**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement physical-unit geometry and bounds validation**
- [ ] **Step 4: Ensure the same resolved transform feeds preview and production**
- [ ] **Step 5: Commit**

### Task 3.4: Implement manual layout and gang-sheet model

**Files:**
- Modify: `crates/veylora-layout/src/placement.rs`
- Create: `crates/veylora-layout/src/gang_sheet.rs`
- Create: `crates/veylora-layout/tests/gang_sheet.rs`

**Interfaces:**

```rust
pub struct GangSheet { pub placements: Vec<Placement>, pub gutters_mm: Margins, pub labels: Vec<LabelSpec> }
pub fn duplicate_placement(sheet: &mut GangSheet, placement: PlacementId, copies: u32, independent: bool) -> Result<Vec<PlacementId>>;
pub fn block_nest(sheet: &GangSheet, constraints: NestConstraints) -> Result<GangSheet>;
```

- [ ] **Step 1: Test copies, clone versus independent placement, gutters, bounds, crop offsets, and selected reprint identity**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement manual placement and deterministic rectangle/block nesting**
- [ ] **Step 4: Add portable bundle references without tying the model to filesystem paths**
- [ ] **Step 5: Commit**

### Gate B: rendering correctness

Before building a polished UI, the headless renderer must pass hidden-RGB, alpha, opaque-white, transform, clear-mask, band-size, and deterministic-output fixtures. The output inspector must be able to show logical planes and all transforms.

---

## Phase 4: separation, masks, screening, and bounded rendering

### Task 4.1: Implement white policy and supplied-mask pipeline

**Files:**
- Create: `crates/veylora-separation/src/white.rs`
- Create: `crates/veylora-separation/src/mask.rs`
- Create: `crates/veylora-separation/src/morphology.rs`
- Create: `crates/veylora-separation/tests/white_policy.rs`

**Interfaces:**

```rust
pub enum WhiteSource { Alpha, Tonal, SuppliedMask, Disabled }
pub enum MaskCombination { Replace, UnionMax, IntersectionMin, Blend { supplied_weight: f32 } }
pub struct WhitePolicy { pub source: WhiteSource, pub combination: MaskCombination, pub curve: CurveId, pub choke_mm: f64, pub spread_mm: f64, pub max_coverage: f32 }
pub fn generate_white(input: &SeparationInput, policy: &WhitePolicy) -> Result<Plane, SeparationError>;
```

- [ ] **Step 1: Test alpha white, supplied-mask replacement/union/intersection, opaque white with zero CMYK, transparent zero-ink, curve behavior, and choke/spread**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement the declared precedence: source, combination, curve, geometry, clear mask, allocation, screening**
- [ ] **Step 4: Verify changing only white policy leaves process coverage unchanged**
- [ ] **Step 5: Commit**

### Task 4.2: Implement knockout, perforation, and final clear mask

**Files:**
- Create: `crates/veylora-separation/src/knockout.rs`
- Create: `crates/veylora-separation/src/perforation.rs`
- Create: `crates/veylora-separation/tests/clear_mask.rs`

**Interfaces:**

```rust
pub struct KnockoutRule { pub target: ColorTarget, pub tolerance: f32, pub hard_clip: f32, pub midtone: f32 }
pub struct PerforationRule { pub diameter_mm: f64, pub spacing_mm: f64, pub eligible: EligibilityMask }
pub fn build_clear_mask(input: &MaskInput, rules: &[MaskRule]) -> Result<ClearMask, SeparationError>;
pub fn enforce_clear_mask(planes: &mut PlaneSet, clear: &ClearMask);
```

- [ ] **Step 1: Test black/near-black, selected color, gradients, antialiasing, thin text, and holes**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement reversible masks and final enforcement over every plane**
- [ ] **Step 4: Verify no later white spread or screening reintroduces coverage**
- [ ] **Step 5: Commit**

### Task 4.3: Implement logical plane representation and white-slot allocation

**Files:**
- Create: `crates/veylora-separation/src/planes.rs`
- Create: `crates/veylora-separation/src/allocation.rs`
- Create: `crates/veylora-separation/tests/allocation.rs`

**Interfaces:**

```rust
pub struct LogicalPlanes { pub c: Plane, pub m: Plane, pub y: Plane, pub k: Plane, pub w: Plane }
pub struct PhysicalPlanes { pub slots: BTreeMap<ProtocolChannel, Plane> }
pub fn allocate_logical_to_physical(logical: &LogicalPlanes, map: &LogicalChannelMap, limits: &AllocationLimits) -> Result<PhysicalPlanes, AllocationError>;
```

- [ ] **Step 1: Test one-white, two-white, unavailable-slot, and allocation-limit cases**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement measured allocation policies without duplicating full white coverage**
- [ ] **Step 4: Record allocation policy in the recipe hash**
- [ ] **Step 5: Commit**

### Task 4.4: Implement deterministic calibration and screening

**Files:**
- Create: `crates/veylora-screen/src/calibration.rs`
- Create: `crates/veylora-screen/src/halftone.rs`
- Create: `crates/veylora-screen/src/lib.rs`
- Create: `crates/veylora-screen/tests/band_equivalence.rs`

**Interfaces:**

```rust
pub struct ChannelCurve { pub samples: Vec<f32> }
pub struct ScreenSpec { pub method: ScreenMethod, pub phase_origin: (i64, i64), pub supported_levels: u16 }
pub fn apply_calibration(planes: &mut PhysicalPlanes, curves: &CalibrationSet, limits: &InkLimits);
pub fn screen_band(plane: &PlaneBand, origin: PageOrigin, spec: &ScreenSpec) -> ScreenedBand;
```

- [ ] **Step 1: Test ramps, limits, phase stability, and banded/unbanded equivalence**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement one deterministic baseline screen and explicit channel limits**
- [ ] **Step 4: Compare different band heights and ensure no seams or phase resets**
- [ ] **Step 5: Commit**

### Task 4.5: Implement bounded band renderer and intermediate artifacts

**Files:**
- Create: `crates/veylora-render/src/bands.rs`
- Create: `crates/veylora-render/src/pipeline.rs`
- Create: `crates/veylora-render/src/artifacts.rs`
- Create: `crates/veylora-render/tests/pipeline_fixtures.rs`

**Interfaces:**

```rust
pub struct RenderRequest { pub snapshot: JobSnapshot, pub cancellation: CancellationToken, pub limits: RenderLimits }
pub struct RenderArtifacts { pub logical_planes: ArtifactRef, pub physical_planes: ArtifactRef, pub screened_planes: ArtifactRef }
pub fn render(request: RenderRequest) -> Result<RenderArtifacts, RenderError>;
```

- [ ] **Step 1: Test cancellation, disk-full, row halos, hidden RGB, clear masks, and band-size equivalence**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement disk-backed band pipeline with explicit halo handling**
- [ ] **Step 4: Record stage hashes and resource statistics**
- [ ] **Step 5: Commit**

### Gate C: inspectable native-ready raster

Require:

- deterministic logical, physical-slot, and screened outputs;
- white policy independent from process coverage;
- clear-mask enforcement over all planes;
- stable output across band sizes;
- bounded memory and cancellation;
- exact stage artifacts that can be inspected without recomputing a different preview.

---

## Phase 5: ET-8550 encoder and Windows transport

### Task 5.1: Implement ET-8550 protocol encoder behind the file sink

**Files:**
- Create: `crates/veylora-encoder/src/et8550.rs`
- Create: `crates/veylora-encoder/src/framing.rs`
- Create: `crates/veylora-encoder/src/lib.rs`
- Create: `crates/veylora-encoder/tests/golden_stream.rs`
- Add: `fixtures/expected/et8550/`
- Add: `docs/bench/et8550-protocol-spec.md`

**Interfaces:**

```rust
pub trait DeviceEncoder { fn encode(&self, input: EncodedPage) -> Result<PrinterArtifact, EncodeError>; }
pub struct Et8550Encoder { pub mode: PrintMode }
pub struct PrinterArtifact { pub path: PathBuf, pub content_hash: ContentHash, pub byte_count: u64, pub metadata: ArtifactMetadata }
```

- [ ] **Step 1: Create golden fixtures from tiny bounded, legally obtained reference evidence where permitted**
- [ ] **Step 2: Write failing tests for framing, page size, channel selection, band order, and end markers**
- [ ] **Step 3: Implement only the protocol fields established by evidence**
- [ ] **Step 4: Reject unsupported mode fields rather than silently ignoring them**
- [ ] **Step 5: Verify deterministic stream hashes and artifact metadata**
- [ ] **Step 6: Commit**

### Task 5.2: Implement Windows spooler transport and fake transport

**Files:**
- Create: `crates/veylora-transport/src/lib.rs`
- Create: `crates/veylora-transport/src/windows.rs`
- Create: `crates/veylora-transport/src/fake.rs`
- Create: `crates/veylora-transport/tests/attempts.rs`

**Interfaces:**

```rust
pub trait PrinterTransport { fn submit(&self, artifact: &PrinterArtifact, observer: &mut dyn SubmissionObserver) -> Result<TransportResult, TransportError>; }
pub trait SubmissionObserver { fn on_bytes(&mut self, count: u64); fn on_status(&mut self, status: TransportStatus); }
```

- [ ] **Step 1: Test fake transport failures before, during, and after writing**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement Windows `StartDocPrinter`/`StartPagePrinter`/`WritePrinter`/`EndPagePrinter`/`EndDocPrinter` adapter in a worker-compatible API**
- [ ] **Step 4: Record bytes written and preserve ambiguous outcomes**
- [ ] **Step 5: Commit**

### Task 5.3: Add native probe commands and first physical submission

**Files:**
- Modify: `crates/veylora-cli/src/main.rs`
- Create: `crates/veylora-cli/src/native.rs`
- Create: `docs/bench/gate-a-results.md`
- Modify: `device-config/et8550/mode-experimental.json`

**Interfaces:**

```text
veylora native render-target --target channel-identity --mode <id>
veylora native inspect --artifact <path>
veylora native submit --artifact <path> --experimental
veylora native record-result --attempt <id> --result <json>
```

- [ ] **Step 1: Add CLI tests that refuse submission without `--experimental` and a verified mode**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Wire render, encode, file artifact, transport, and attempt persistence**
- [ ] **Step 4: Run the physical channel/geometry/overlap target**
- [ ] **Step 5: Record observations, failures, and exact setup hashes**
- [ ] **Step 6: Promote the mode only after physical acceptance**
- [ ] **Step 7: Commit the bench evidence separately from code**

### Gate D: native backend proof

Do not proceed to desktop production workflow unless the actual printer demonstrates:

- independent required channel addressing;
- intended process/white deposition order;
- correct mirror/orientation and physical scale;
- usable feed and ejection on chosen film;
- repeatable output after restart and fresh media;
- documented handling of partial submission and cancellation.

If the gate fails, keep the encoder and probe experimental and continue only with a specifically bounded investigation.

---

## Phase 6: calibration, recipes, and quality workflow

### Task 6.1: Implement calibration and recipe storage

**Files:**
- Create: `crates/veylora-color/src/calibration.rs`
- Modify: `crates/veylora-domain/src/recipe.rs`
- Create: `crates/veylora-persistence/src/recipes.rs`
- Create: `crates/veylora-color/tests/calibration.rs`

**Interfaces:**

```rust
pub struct CalibrationRecord { pub channels: Vec<ChannelCurve>, pub limits: InkLimits, pub white_slots: WhiteCalibration, pub measurement: MeasurementProvenance }
pub enum RecipeStatus { Experimental, Validated, Superseded }
pub fn validate_recipe(recipe: &PrintRecipe, setup: &DeviceSetup, mode: &PrintMode) -> Vec<Finding>;
```

- [ ] **Step 1: Test incompatible profile/mode/setup/white-policy combinations**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement immutable calibration records and recipe compatibility checks**
- [ ] **Step 4: Record external ArgyllCMS measurement references without claiming N-color profile support**
- [ ] **Step 5: Commit**

### Task 6.2: Implement target export and measurement import

**Files:**
- Create: `crates/veylora-diagnostics/src/charts.rs`
- Create: `crates/veylora-color/src/measurements.rs`
- Create: `crates/veylora-color/tests/measurement_import.rs`

- [ ] **Step 1: Test target IDs, patch counts, missing patches, and malformed measurements**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement chart metadata and import adapters for the chosen external measurement format**
- [ ] **Step 4: Verify holdout patches are excluded from calibration fitting**
- [ ] **Step 5: Commit**

### Task 6.3: Define physical quality protocol and baseline records

**Files:**
- Create: `docs/bench/quality-protocol.md`
- Create: `docs/bench/quality-baseline-template.md`
- Create: `fixtures/physical/README.md`

- [ ] **Step 1: Define measured baseline procedure for nozzle health, ramps, registration, wet load, press, cure, peel, stretch, and wash**
- [ ] **Step 2: Record a commercial/reference print if available**
- [ ] **Step 3: Freeze tolerances only after baseline measurement**
- [ ] **Step 4: Store sample IDs, supplies, instrument conditions, and photographs/measurements**
- [ ] **Step 5: Commit protocol and baseline records**

### Gate E: personal production recipe

A recipe becomes validated only when repeated pressed samples meet frozen tolerances across sessions. Until then, the application must label it experimental or unprofiled.

---

## Phase 7: preflight, queue, reprints, and diagnostics

### Task 7.1: Implement preflight orchestration

**Files:**
- Create: `crates/veylora-domain/src/preflight_rules.rs`
- Create: `crates/veylora-app-services/src/preflight.rs`
- Create: `crates/veylora-app-services/Cargo.toml`
- Create: `crates/veylora-app-services/tests/preflight.rs`

**Interfaces:**

```rust
pub struct PreflightRequest { pub snapshot: JobSnapshot, pub setup: DeviceSetup, pub mode: PrintMode, pub recipe: PrintRecipe, pub resources: ResourceState }
pub fn run_preflight(request: PreflightRequest) -> PreflightReport;
```

- [ ] **Step 1: Test missing assets, unsupported profiles, unverified setup, incompatible recipe, clipping, limits, disk space, and unsupported controls**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement blocking/confirmation/informational findings**
- [ ] **Step 4: Verify blocking findings prevent render or submission**
- [ ] **Step 5: Commit**

### Task 7.2: Implement durable lifecycle and submission attempts

**Files:**
- Create: `crates/veylora-domain/src/lifecycle.rs`
- Modify: `crates/veylora-persistence/src/jobs.rs`
- Create: `crates/veylora-app-services/src/submission.rs`
- Create: `crates/veylora-app-services/tests/fault_injection.rs`

**Interfaces:**

```rust
pub enum JobState { Draft, PreflightFailed, Held, Rendering, Ready, Submitting, Submitted, Ambiguous, NeedsReview, Cancelled, Failed, OperatorConfirmedFinished }
pub fn transition(current: JobState, event: LifecycleEvent) -> Result<JobState, LifecycleError>;
```

- [ ] **Step 1: Test every legal/illegal state transition and crash boundary**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement transactional state updates and append-only attempts**
- [ ] **Step 4: Test restart reconstruction, disk-full, timeout, and no automatic duplicate**
- [ ] **Step 5: Commit**

### Task 7.3: Implement diagnostics and support bundles

**Files:**
- Create: `crates/veylora-diagnostics/src/report.rs`
- Create: `crates/veylora-diagnostics/src/support_bundle.rs`
- Create: `crates/veylora-diagnostics/tests/redaction.rs`

- [ ] **Step 1: Test reports with and without artwork, redacted paths, hashes, and diagnostic targets**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement compact job/attempt reports and safe support archives**
- [ ] **Step 4: Verify archive extraction prevents path traversal and does not upload data**
- [ ] **Step 5: Commit**

---

## Phase 8: headless daily-use CLI and Tauri desktop application

### Task 8.1: Complete headless CLI before UI

**Files:**
- Modify: `crates/veylora-cli/src/main.rs`
- Create: `crates/veylora-cli/src/commands.rs`
- Create: `crates/veylora-cli/tests/workflows.rs`

Commands:

```text
veylora setup create
veylora setup verify
veylora target generate
veylora asset inspect
veylora job create
veylora job preflight
veylora job render
veylora artifact inspect
veylora job submit
veylora job review
veylora job reprint
veylora report export
```

- [ ] **Step 1: Add end-to-end file-sink workflow tests**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Wire commands through application services rather than duplicating domain logic**
- [ ] **Step 4: Verify a complete supported PNG-to-artifact workflow without a printer**
- [ ] **Step 5: Commit**

### Task 8.2: Create the Tauri 2 desktop UI shell with Svelte and Bits UI

**Files:**
- Create: `crates/veylora-app/Cargo.toml`
- Create: `crates/veylora-app/src/main.rs`
- Create: `crates/veylora-app/src/state.rs`
- Create: `app/package.json`
- Create: `app/pnpm-lock.yaml`
- Create: `app/vite.config.ts`
- Create: `app/src/`
- Create: `app/tests/`

The Tauri shell and Svelte frontend must consume typed application-service commands and read-model projections. They must not call image, separation, encoder, persistence, or Windows APIs directly. Bits UI primitives must be wrapped by Veylora-owned components when product semantics, validation, styling, or accessibility behavior are added.

The UI-first workflow may begin with mocked application services and clearly labeled fixture data. It must not claim that fixture data or simulated previews represent physical printer output. The frontend uses Vite+ commands for development, checking, testing, building, and package-management delegation. Vite+ detects the project's declared pnpm package manager, while `package.json` and the lockfile remain the dependency source of truth. Pin the Vite+ and pnpm versions used by local development and CI before the first frontend implementation commit.

Initial screens:

- Setup and channel verification.
- Device mode and recipe selection.
- Artwork import and metadata warnings.
- Physical layout and gang sheet.
- White policy and mask controls.
- Garment/film/plane/screened inspector.
- Preflight report.
- Queue, attempt history, and review state.
- Diagnostic target/report export.

- [ ] **Step 1: Test navigation and command enablement from capability state**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement the Tauri 2 shell and Svelte frontend with mocked application services**
- [ ] **Step 4: Connect one file-sink workflow**
- [ ] **Step 5: Verify unsupported controls are disabled or clearly marked**
- [ ] **Step 6: Commit**

The UI prototype and shell may be implemented before the native ET-8550 gate. Backend integration remains gated, while the UX uses `Simulation`, `Fixture data`, `Experimental`, and `Not physical output` labels where appropriate.

### Task 8.3: Implement UI inspection and preflight presentation

**Files:**
- Modify: `crates/veylora-app/src/screens/`
- Create: `crates/veylora-app/src/view_models.rs`
- Create: `crates/veylora-app/tests/preflight_presentation.rs`

- [ ] **Step 1: Test that every blocking finding has a visible correction message**
- [ ] **Step 2: Implement logical/physical/screened plane toggles and metadata display**
- [ ] **Step 3: Implement hold/inspect/release and ambiguous-review actions**
- [ ] **Step 4: Verify the UI never displays a simulated preview as final output**
- [ ] **Step 5: Commit**

---

## Phase 9: portable jobs, installation, and future distribution readiness

### Task 9.1: Implement bundled job archives

**Files:**
- Create: `crates/veylora-persistence/src/bundle.rs`
- Create: `crates/veylora-persistence/tests/bundle_roundtrip.rs`
- Create: `docs/job-format.md`

- [ ] **Step 1: Test bundle round trips, moved paths, missing assets, checksums, and incompatible schema versions**
- [ ] **Step 2: Verify failure**
- [ ] **Step 3: Implement path-independent archive format with safe extraction**
- [ ] **Step 4: Add schema version and migration dispatch**
- [ ] **Step 5: Commit**

### Task 9.2: Add clean-machine packaging and update safety

**Files:**
- Create: `installer/README.md`
- Create: `installer/windows/`
- Create: `scripts/package-release.ps1`
- Create: `scripts/verify-release.ps1`
- Create: `docs/compatibility-matrix.md`
- Create: `docs/release-process.md`

- [ ] **Step 1: Define supported Windows version, runtime prerequisites, device setup prerequisites, and rollback behavior**
- [ ] **Step 2: Build a clean-machine installation test**
- [ ] **Step 3: Verify the package includes no development-tool dependency**
- [ ] **Step 4: Add release hash/signature verification when the chosen installer supports it**
- [ ] **Step 5: Test update refusal during active submission and rollback after failed update**
- [ ] **Step 6: Commit**

### Task 9.3: Complete dependency, profile, font, and license inventory

**Files:**
- Create: `THIRD_PARTY_NOTICES.md`
- Create: `docs/dependency-inventory.md`
- Create: `docs/profile-and-font-licensing.md`
- Create: `scripts/check-licenses.ps1`

- [ ] **Step 1: Inventory Rust crates, native libraries, external tools, profiles, fonts, and fixture assets**
- [ ] **Step 2: Record licenses and redistribution permissions**
- [ ] **Step 3: Fail the release check when required notices are absent**
- [ ] **Step 4: Commit inventory and release checks**

### Gate F: distribution-ready beta

A distribution candidate must install on a clean supported Windows machine, import a bundled job, reject incompatible device/recipe combinations, render a diagnostic target, export a redacted support bundle, and recover safely from an interrupted update.

---

## Cross-cutting test strategy

### Unit tests

Use deterministic tests for geometry, alpha, color policy, white generation, mask algebra, curves, calibration, limits, screen phase, channel allocation, state transitions, schema migrations, and archive safety.

### Property tests

Generate arbitrary bounded alpha masks and placements to verify:

- clear-mask enforcement never increases coverage;
- physical dimensions remain stable through transforms;
- mirror twice returns to original orientation;
- band concatenation matches full-frame rendering within the declared tolerance;
- no output exists outside the printable box;
- changing only white policy leaves process coverage unchanged;
- job hashes change when any output-affecting input changes.

### Golden tests

Store small, reviewable expected plane files and native stream artifacts for:

- channel identity;
- overlap order;
- alpha/hidden RGB;
- opaque white;
- choke/spread;
- white allocation;
- screen phase;
- encoder framing;
- unsupported settings.

Golden fixtures must include metadata explaining their source, mode, and whether they are synthetic or captured reference artifacts.

### Fault injection

Exercise failure at decode, render, disk allocation, artifact write, encoder write, spooler submission, cancellation, database commit, restart, and update. Assert explicit state transitions and no silent duplicate submission.

### Physical tests

Maintain a bench log for each test with setup hash, mode hash, recipe hash, media, film, ink, nozzle status, target ID, printer status, photos/measurements, and operator result. A physical result cannot be inferred from a passing software test.

### Verification commands

At every task:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

At rendering milestones:

```text
cargo test --workspace --features fixture-tests
cargo run -p veylora-cli -- artifact inspect --help
```

At Windows transport milestones, run the same suite on a supported Windows machine and add the native integration tests only when the printer is connected and a physical target is staged.

---

## Implementation order and stop rules

1. Workspace and domain primitives.
2. Device records and diagnostics.
3. Protocol investigation CLI.
4. PrintMode and immutable jobs.
5. PNG/color/layout core.
6. White/mask/screen/band renderer.
7. Output inspector and file sink.
8. ET-8550 encoder.
9. Windows transport.
10. Gate A/D native physical proof.
11. Calibration and quality recipe.
12. Preflight and crash-safe queue.
13. Headless daily-use CLI.
14. Desktop UI.
15. Portable bundles and distribution packaging.

Stop or re-plan at any point when:

- independent native channel control cannot be established;
- the selected media path cannot feed reliably;
- final output cannot be inspected sufficiently to diagnose failures;
- physical results cannot meet the frozen baseline for the selected recipe;
- a dependency cannot be legally redistributed for a future distribution build.

A working layout editor is not permission to bypass a failed native backend gate. A successful native patch is not permission to claim production readiness without repeated physical output.

## Commit and review checkpoints

Use one focused commit per task. Before merging each phase:

1. Review the task's changed files and public interfaces.
2. Run formatting, lint, unit, property, golden, and relevant integration tests.
3. Update the bench record or fixture manifest when physical evidence changes a mode.
4. Confirm no UI-only assumption leaked into the headless core.
5. Confirm all output-affecting inputs are represented in hashes and snapshots.
6. Record the gate result and unresolved risks in the phase document.

The plan is complete when Veylora RIP can produce, inspect, and safely submit a native printer-ready artifact for one verified ET-8550 configuration, then execute a repeatable personal production workflow with measured quality and recoverable jobs.
