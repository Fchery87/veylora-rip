//! Offline ICC color transform boundary.
//!
//! This module is the single authorized entry point for converting pixel data
//! from a source color space to Veylora's working/device color space. It wraps
//! a real ICC engine (Firefox's `qcms`, pure Rust, MIT-licensed) rather than
//! hand-rolled matrix math and keeps the following invariants:
//!
//! * The transform is bounded and deterministic. All configuration that can
//!   change the output is captured in [`TransformConfig`] and recorded in
//!   [`TransformReceipt::config_hash`].
//! * The source profile is identified by the SHA-256 of its exact bytes. The
//!   same bytes produce the same hash across runs, independent of file names.
//! * Rendering intent, black-point compensation, and the untagged-input policy
//!   are explicit. There is no implicit assumption that untagged data is sRGB
//!   unless the caller requests [`UntaggedPolicy::AssumeSrgb`].
//! * Unsupported or unparsable profiles produce [`ColorError::UnsupportedProfile`]
//!   with [`crate::errors::ErrorCode::UnsupportedProfile`]. The engine never
//!   silently approximates.
//! * A buffer that has already been transformed cannot be transformed again by
//!   this boundary without an explicit reset. See [`PixelBuffer`].
//!
//! Scope note: this release wires the transform API, boundary policy, and
//! source-side profile handling. The working/output profile is currently fixed
//! to the engine's precomputed sRGB target, which is the first validated
//! destination for the ET-8550 preview path. CMYK output profiles for the
//! device raster pipeline, and black-point compensation inside the engine
//! itself, are explicit follow-ups documented in the project ADR; this module
//! surfaces [`BlackPointCompensation`] in the configuration hash so the
//! setting is captured in provenance even though the current engine applies
//! only relative/absolute colorimetric behavior natively.

use crate::errors::ErrorCode;
use crate::provenance::ContentHash;
use serde::{Deserialize, Serialize};

/// ICC rendering intent. Values match the ICC specification numbering so they
/// can be serialized stably into provenance records.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

impl RenderingIntent {
    const fn as_qcms(self) -> qcms::Intent {
        match self {
            Self::Perceptual => qcms::Intent::Perceptual,
            Self::RelativeColorimetric => qcms::Intent::RelativeColorimetric,
            Self::Saturation => qcms::Intent::Saturation,
            Self::AbsoluteColorimetric => qcms::Intent::AbsoluteColorimetric,
        }
    }

    const fn tag(self) -> &'static [u8] {
        match self {
            Self::Perceptual => b"intent:perceptual",
            Self::RelativeColorimetric => b"intent:relative_colorimetric",
            Self::Saturation => b"intent:saturation",
            Self::AbsoluteColorimetric => b"intent:absolute_colorimetric",
        }
    }
}

/// Black-point compensation policy. Recorded in provenance so that changing
/// this setting produces a distinct configuration hash even when the
/// underlying ICC engine applies black-point behavior implicitly for the
/// chosen rendering intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlackPointCompensation {
    /// Preserve absolute black. Used with absolute colorimetric intent.
    Off,
    /// Map source black to destination black. Default for relative colorimetric
    /// and perceptual in most production RIPs.
    On,
}

/// Policy for data that arrives without an embedded or declared source profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UntaggedPolicy {
    /// Refuse the input. Produces [`ColorError::UntaggedInput`].
    Reject,
    /// Treat the input as sRGB. The caller must have verified this is
    /// appropriate for the source. The engine's sRGB profile is used and its
    /// identity is included in the configuration hash.
    AssumeSrgb,
}

/// Pixel data type passed through the transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PixelFormat {
    /// 8-bit per channel, red/green/blue/alpha, straight (not premultiplied).
    Rgba8,
    /// 8-bit per channel, red/green/blue.
    Rgb8,
}

impl PixelFormat {
    const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgba8 => 4,
            Self::Rgb8 => 3,
        }
    }

    const fn qcms_data_type(self) -> qcms::DataType {
        match self {
            Self::Rgba8 => qcms::DataType::RGBA8,
            Self::Rgb8 => qcms::DataType::RGB8,
        }
    }

    const fn tag(self) -> &'static [u8] {
        match self {
            Self::Rgba8 => b"format:rgba8",
            Self::Rgb8 => b"format:rgb8",
        }
    }
}

/// A source ICC profile identified by its exact bytes.
#[derive(Clone, Debug)]
pub struct SourceProfile {
    bytes: Vec<u8>,
    hash: ContentHash,
}

impl SourceProfile {
    /// Parse an ICC profile from its raw bytes. The bytes must be a self-
    /// contained ICC v2 or v4 profile recognized by the engine; otherwise
    /// [`ColorError::UnsupportedProfile`] is returned.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, ColorError> {
        // Parse once at construction time to fail fast. The returned Profile
        // is not retained because qcms Profiles are not Clone; the Transform
        // build step reparses the same bytes, which is deterministic and keeps
        // the buffer the single owner.
        if qcms::Profile::new_from_slice(&bytes, false).is_none() {
            return Err(ColorError::UnsupportedProfile {
                detail: "Source ICC profile could not be parsed by the ICC engine.",
            });
        }
        let hash = ContentHash::digest(&bytes);
        Ok(Self { bytes, hash })
    }

    /// SHA-256 of the exact source profile bytes.
    pub fn hash(&self) -> ContentHash {
        self.hash
    }

    /// Raw ICC bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Configuration for a single transform run. Equal configurations produce
/// equal [`TransformConfig::config_hash`] values.
#[derive(Clone, Debug)]
pub struct TransformConfig {
    pub intent: RenderingIntent,
    pub black_point_compensation: BlackPointCompensation,
    pub untagged_policy: UntaggedPolicy,
    pub pixel_format: PixelFormat,
}

impl TransformConfig {
    /// Deterministic hash of the configuration together with a source-profile
    /// hash, used to key provenance records.
    pub fn config_hash(&self, source_hash: ContentHash) -> ContentHash {
        let mut bytes = Vec::with_capacity(128);
        bytes.extend_from_slice(b"veylora-color-transform-v1\0");
        bytes.extend_from_slice(self.intent.tag());
        bytes.push(0);
        bytes.extend_from_slice(match self.black_point_compensation {
            BlackPointCompensation::Off => b"bpc:off",
            BlackPointCompensation::On => b"bpc:on",
        });
        bytes.push(0);
        bytes.extend_from_slice(match self.untagged_policy {
            UntaggedPolicy::Reject => b"untagged:reject",
            UntaggedPolicy::AssumeSrgb => b"untagged:assume_srgb",
        });
        bytes.push(0);
        bytes.extend_from_slice(self.pixel_format.tag());
        bytes.push(0);
        bytes.extend_from_slice(b"source:");
        let source_hex = format!("{source_hash}");
        bytes.extend_from_slice(source_hex.as_bytes());
        ContentHash::digest(&bytes)
    }
}

/// Error cases exposed by the color boundary.
#[derive(Debug, PartialEq, Eq)]
pub enum ColorError {
    /// The source profile could not be parsed or is not supported by the
    /// engine (e.g. unsupported ICC flavor).
    UnsupportedProfile { detail: &'static str },
    /// The caller passed untagged data while the configuration's untagged
    /// policy is [`UntaggedPolicy::Reject`].
    UntaggedInput,
    /// The engine refused to build a transform between the given source and
    /// the configured working profile.
    UnsupportedTransform { detail: &'static str },
    /// The buffer length is not a multiple of the pixel stride.
    InvalidBufferLength { len: usize, stride: usize },
    /// The buffer has already been transformed by this boundary and
    /// [`PixelBuffer::mark_fresh`] was not called.
    AlreadyTransformed,
}

impl ColorError {
    /// Map to the stable domain error code vocabulary.
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::UnsupportedProfile { .. }
            | Self::UntaggedInput
            | Self::UnsupportedTransform { .. } => ErrorCode::UnsupportedProfile,
            Self::InvalidBufferLength { .. } | Self::AlreadyTransformed => {
                ErrorCode::UnsupportedInput
            }
        }
    }
}

impl std::fmt::Display for ColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedProfile { detail } => {
                write!(f, "unsupported source profile: {detail}")
            }
            Self::UntaggedInput => f.write_str(
                "untagged input and untagged policy is reject; attach a source profile or opt into AssumeSrgb explicitly",
            ),
            Self::UnsupportedTransform { detail } => {
                write!(f, "unsupported transform: {detail}")
            }
            Self::InvalidBufferLength { len, stride } => write!(
                f,
                "pixel buffer length {len} is not a multiple of {stride} bytes per pixel",
            ),
            Self::AlreadyTransformed => f.write_str(
                "pixel buffer has already been color-transformed by this boundary",
            ),
        }
    }
}

impl std::error::Error for ColorError {}

/// A pixel buffer whose color state is tracked. Prevents accidental double
/// transforms through this boundary: once [`ColorTransform::apply_to`] has
/// succeeded, the buffer is marked as transformed and a subsequent apply
/// returns [`ColorError::AlreadyTransformed`] unless [`Self::mark_fresh`] is
/// called to acknowledge that the buffer represents new source data.
#[derive(Debug, Clone)]
pub struct PixelBuffer {
    data: Vec<u8>,
    format: PixelFormat,
    transformed: bool,
}

impl PixelBuffer {
    pub fn new(data: Vec<u8>, format: PixelFormat) -> Result<Self, ColorError> {
        let stride = format.bytes_per_pixel();
        if !data.len().is_multiple_of(stride) {
            return Err(ColorError::InvalidBufferLength {
                len: data.len(),
                stride,
            });
        }
        Ok(Self {
            data,
            format,
            transformed: false,
        })
    }

    pub fn format(&self) -> PixelFormat {
        self.format
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn is_transformed(&self) -> bool {
        self.transformed
    }

    /// Acknowledge that this buffer now holds new, untransformed source data
    /// (for example, a new tile) and clear the transformed flag.
    pub fn mark_fresh(&mut self) {
        self.transformed = false;
    }

    /// Consume the buffer, returning its bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

/// Immutable, inspectable record of a transform application. Suitable for
/// inclusion in a job snapshot without copying pixel data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformReceipt {
    pub source_profile_hash: ContentHash,
    pub config_hash: ContentHash,
    pub intent: RenderingIntent,
    pub black_point_compensation: BlackPointCompensation,
    pub untagged_policy: UntaggedPolicy,
    pub pixel_format: PixelFormat,
    pub pixel_count: u64,
    /// Short, stable identifier of the ICC engine that performed the work.
    pub engine: String,
}

/// A built, bounded transform that can be applied to buffers. Deterministic:
/// building the same transform twice from the same inputs and applying it to
/// the same bytes produces byte-identical output within a single engine
/// version.
pub struct ColorTransform {
    transform: qcms::Transform,
    source_hash: ContentHash,
    config: TransformConfig,
}

impl std::fmt::Debug for ColorTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColorTransform")
            .field("source_hash", &self.source_hash)
            .field("config", &self.config)
            .field("engine", &ENGINE_ID)
            .finish()
    }
}

impl ColorTransform {
    /// Build a transform from a tagged source profile to Veylora's working
    /// sRGB target. Returns [`ColorError::UnsupportedTransform`] if the engine
    /// cannot bridge the source to the working profile.
    pub fn from_source(
        source: &SourceProfile,
        config: TransformConfig,
    ) -> Result<Self, ColorError> {
        let input = qcms::Profile::new_from_slice(source.bytes(), false).ok_or(
            ColorError::UnsupportedProfile {
                detail: "Source ICC profile reparse failed.",
            },
        )?;
        let mut output = qcms::Profile::new_sRGB();
        output.precache_output_transform();
        let transform = qcms::Transform::new(
            &input,
            &output,
            config.pixel_format.qcms_data_type(),
            config.intent.as_qcms(),
        )
        .ok_or(ColorError::UnsupportedTransform {
            detail: "ICC engine refused to build the requested transform.",
        })?;
        Ok(Self {
            transform,
            source_hash: source.hash(),
            config,
        })
    }

    /// Build a transform for untagged input according to the configured
    /// policy. Returns [`ColorError::UntaggedInput`] if the policy forbids it.
    pub fn from_untagged(config: TransformConfig) -> Result<Self, ColorError> {
        match config.untagged_policy {
            UntaggedPolicy::Reject => Err(ColorError::UntaggedInput),
            UntaggedPolicy::AssumeSrgb => {
                let input = qcms::Profile::new_sRGB();
                let mut output = qcms::Profile::new_sRGB();
                output.precache_output_transform();
                let transform = qcms::Transform::new(
                    &input,
                    &output,
                    config.pixel_format.qcms_data_type(),
                    config.intent.as_qcms(),
                )
                .ok_or(ColorError::UnsupportedTransform {
                    detail: "ICC engine refused to build sRGB->sRGB transform.",
                })?;
                // Use the hash of the engine's declared sRGB identity token so
                // that two independent runs that both took the untagged path
                // produce the same provenance.
                let source_hash = ContentHash::digest(b"veylora-untagged-srgb-v1");
                Ok(Self {
                    transform,
                    source_hash,
                    config,
                })
            }
        }
    }

    /// Apply this transform to `buffer` in place. The buffer is marked as
    /// transformed on success, and a second apply without
    /// [`PixelBuffer::mark_fresh`] returns [`ColorError::AlreadyTransformed`].
    pub fn apply_to(&self, buffer: &mut PixelBuffer) -> Result<TransformReceipt, ColorError> {
        if buffer.format != self.config.pixel_format {
            return Err(ColorError::InvalidBufferLength {
                len: buffer.data.len(),
                stride: self.config.pixel_format.bytes_per_pixel(),
            });
        }
        if buffer.transformed {
            return Err(ColorError::AlreadyTransformed);
        }
        let stride = self.config.pixel_format.bytes_per_pixel();
        if !buffer.data.len().is_multiple_of(stride) {
            return Err(ColorError::InvalidBufferLength {
                len: buffer.data.len(),
                stride,
            });
        }
        self.transform.apply(&mut buffer.data);
        buffer.transformed = true;
        Ok(TransformReceipt {
            source_profile_hash: self.source_hash,
            config_hash: self.config.config_hash(self.source_hash),
            intent: self.config.intent,
            black_point_compensation: self.config.black_point_compensation,
            untagged_policy: self.config.untagged_policy,
            pixel_format: self.config.pixel_format,
            pixel_count: (buffer.data.len() / stride) as u64,
            engine: ENGINE_ID.to_string(),
        })
    }

    /// Stable identifier of the source profile this transform was built from.
    pub fn source_hash(&self) -> ContentHash {
        self.source_hash
    }

    /// Configuration this transform was built with.
    pub fn config(&self) -> &TransformConfig {
        &self.config
    }
}

/// Short, stable identifier of the ICC engine used.
pub const ENGINE_ID: &str = "qcms-0.3";

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal byte sequence that is *not* a valid ICC profile. Used to
    /// confirm the engine rejects garbage rather than silently approximating.
    const GARBAGE_PROFILE: &[u8] = b"not an icc profile";

    /// Deterministic SHA-256 identity of a hypothetical source profile. The
    /// color-boundary configuration hash depends only on this value, not on
    /// the engine's internal parse tree, so tests can exercise the hashing
    /// contract without bundling an ICC fixture.
    fn stub_source_hash() -> ContentHash {
        ContentHash::digest(b"veylora-test-source-profile-v1")
    }

    #[test]
    fn unsupported_profile_bytes_are_rejected() {
        let err = SourceProfile::from_bytes(GARBAGE_PROFILE.to_vec()).unwrap_err();
        assert!(matches!(err, ColorError::UnsupportedProfile { .. }));
        assert_eq!(err.code(), ErrorCode::UnsupportedProfile);
    }

    #[test]
    fn config_hash_changes_with_intent_and_bpc_and_untagged_policy() {
        let source_hash = stub_source_hash();
        let base = TransformConfig {
            intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: BlackPointCompensation::On,
            untagged_policy: UntaggedPolicy::Reject,
            pixel_format: PixelFormat::Rgba8,
        };
        let h0 = base.config_hash(source_hash);
        let mut alt = base.clone();
        alt.intent = RenderingIntent::Perceptual;
        let h1 = alt.config_hash(source_hash);
        let mut alt2 = base.clone();
        alt2.black_point_compensation = BlackPointCompensation::Off;
        let h2 = alt2.config_hash(source_hash);
        let mut alt3 = base.clone();
        alt3.untagged_policy = UntaggedPolicy::AssumeSrgb;
        let h3 = alt3.config_hash(source_hash);
        assert_ne!(h0, h1);
        assert_ne!(h0, h2);
        assert_ne!(h0, h3);
        assert_ne!(h1, h2);
    }

    #[test]
    fn untagged_reject_returns_untagged_error() {
        let config = TransformConfig {
            intent: RenderingIntent::Perceptual,
            black_point_compensation: BlackPointCompensation::On,
            untagged_policy: UntaggedPolicy::Reject,
            pixel_format: PixelFormat::Rgba8,
        };
        let err = ColorTransform::from_untagged(config).unwrap_err();
        assert_eq!(err, ColorError::UntaggedInput);
        assert_eq!(err.code(), ErrorCode::UnsupportedProfile);
    }

    #[test]
    fn untagged_assume_srgb_builds_and_applies() {
        let config = TransformConfig {
            intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: BlackPointCompensation::On,
            untagged_policy: UntaggedPolicy::AssumeSrgb,
            pixel_format: PixelFormat::Rgba8,
        };
        let transform = ColorTransform::from_untagged(config).expect("sRGB->sRGB builds");
        let mut buffer =
            PixelBuffer::new(vec![10, 20, 30, 255, 200, 100, 50, 128], PixelFormat::Rgba8).unwrap();
        let receipt = transform.apply_to(&mut buffer).unwrap();
        assert_eq!(receipt.engine, ENGINE_ID);
        assert_eq!(receipt.pixel_count, 2);
        assert_eq!(receipt.intent, RenderingIntent::RelativeColorimetric);
    }

    #[test]
    fn apply_marks_buffer_and_rejects_double_transform() {
        let config = TransformConfig {
            intent: RenderingIntent::Perceptual,
            black_point_compensation: BlackPointCompensation::On,
            untagged_policy: UntaggedPolicy::AssumeSrgb,
            pixel_format: PixelFormat::Rgba8,
        };
        let transform = ColorTransform::from_untagged(config).unwrap();
        let mut buffer =
            PixelBuffer::new(vec![0, 0, 0, 255, 255, 255, 255, 255], PixelFormat::Rgba8).unwrap();
        transform.apply_to(&mut buffer).unwrap();
        assert!(buffer.is_transformed());
        let err = transform.apply_to(&mut buffer).unwrap_err();
        assert_eq!(err, ColorError::AlreadyTransformed);
        buffer.mark_fresh();
        transform
            .apply_to(&mut buffer)
            .expect("fresh buffer can be transformed again");
    }

    #[test]
    fn transform_is_deterministic_for_identical_inputs() {
        let config = TransformConfig {
            intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: BlackPointCompensation::On,
            untagged_policy: UntaggedPolicy::AssumeSrgb,
            pixel_format: PixelFormat::Rgba8,
        };
        let t1 = ColorTransform::from_untagged(config.clone()).unwrap();
        let t2 = ColorTransform::from_untagged(config).unwrap();
        let input: Vec<u8> = (0..64).collect();
        let mut b1 = PixelBuffer::new(input.clone(), PixelFormat::Rgba8).unwrap();
        let mut b2 = PixelBuffer::new(input, PixelFormat::Rgba8).unwrap();
        let r1 = t1.apply_to(&mut b1).unwrap();
        let r2 = t2.apply_to(&mut b2).unwrap();
        assert_eq!(b1.as_bytes(), b2.as_bytes());
        assert_eq!(r1.config_hash, r2.config_hash);
        assert_eq!(r1.source_profile_hash, r2.source_profile_hash);
    }

    #[test]
    fn mismatched_pixel_format_errors() {
        let config = TransformConfig {
            intent: RenderingIntent::Perceptual,
            black_point_compensation: BlackPointCompensation::On,
            untagged_policy: UntaggedPolicy::AssumeSrgb,
            pixel_format: PixelFormat::Rgba8,
        };
        let transform = ColorTransform::from_untagged(config).unwrap();
        let mut buffer = PixelBuffer::new(vec![0, 0, 0, 255, 255, 255], PixelFormat::Rgb8).unwrap();
        let err = transform.apply_to(&mut buffer).unwrap_err();
        assert!(matches!(err, ColorError::InvalidBufferLength { .. }));
    }

    #[test]
    fn invalid_buffer_length_errors() {
        let err = PixelBuffer::new(vec![0, 1, 2], PixelFormat::Rgba8).unwrap_err();
        assert!(matches!(
            err,
            ColorError::InvalidBufferLength { len: 3, stride: 4 }
        ));
    }

    #[test]
    fn receipt_serializes_stably() {
        let config = TransformConfig {
            intent: RenderingIntent::Perceptual,
            black_point_compensation: BlackPointCompensation::On,
            untagged_policy: UntaggedPolicy::AssumeSrgb,
            pixel_format: PixelFormat::Rgba8,
        };
        let transform = ColorTransform::from_untagged(config).unwrap();
        let mut buffer = PixelBuffer::new(vec![0; 16], PixelFormat::Rgba8).unwrap();
        let receipt = transform.apply_to(&mut buffer).unwrap();
        let json = serde_json::to_string(&receipt).unwrap();
        let round: TransformReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(receipt, round);
    }
}
