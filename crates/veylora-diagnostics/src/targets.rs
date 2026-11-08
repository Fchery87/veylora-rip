use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Generator layout limits, not printer media capabilities.
pub const MIN_SIZE_MM: f64 = 50.0;
pub const MAX_SIZE_MM: f64 = 1000.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SizeWire")]
pub struct PhysicalSize {
    width_mm: f64,
    height_mm: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SizeWire {
    width_mm: f64,
    height_mm: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSize;
impl fmt::Display for InvalidSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "target dimensions must be finite and between {MIN_SIZE_MM} and {MAX_SIZE_MM} mm inclusive")
    }
}
impl Error for InvalidSize {}
impl TryFrom<SizeWire> for PhysicalSize {
    type Error = InvalidSize;
    fn try_from(value: SizeWire) -> Result<Self, Self::Error> {
        Self::new(value.width_mm, value.height_mm)
    }
}
impl PhysicalSize {
    pub fn new(width_mm: f64, height_mm: f64) -> Result<Self, InvalidSize> {
        if [width_mm, height_mm]
            .iter()
            .all(|v| v.is_finite() && (MIN_SIZE_MM..=MAX_SIZE_MM).contains(v))
        {
            Ok(Self {
                width_mm,
                height_mm,
            })
        } else {
            Err(InvalidSize)
        }
    }
    pub fn width_mm(self) -> f64 {
        self.width_mm
    }
    pub fn height_mm(self) -> f64 {
        self.height_mm
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    ChannelIdentity,
    Geometry,
    Registration,
    WhiteRamp,
    WetLoad,
    AlphaEdge,
    Screening,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "TargetWire")]
pub struct DiagnosticTarget {
    schema: String,
    version: u32,
    provenance: String,
    kind: TargetKind,
    size: PhysicalSize,
    objects: Vec<TargetObject>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetWire {
    schema: String,
    version: u32,
    provenance: String,
    kind: TargetKind,
    size: PhysicalSize,
    objects: Vec<TargetObject>,
}

impl TryFrom<TargetWire> for DiagnosticTarget {
    type Error = &'static str;
    fn try_from(wire: TargetWire) -> Result<Self, Self::Error> {
        let target = generate_target(wire.kind, wire.size);
        // This format is a generated target, not an arbitrary artwork interchange.
        // Reject edits instead of silently accepting inconsistent semantics or geometry.
        if wire.schema != target.schema
            || wire.version != target.version
            || wire.provenance != target.provenance
            || wire.objects != target.objects
        {
            return Err("unsupported or modified diagnostic target geometry/version");
        }
        Ok(target)
    }
}
impl DiagnosticTarget {
    pub fn size(&self) -> PhysicalSize {
        self.size
    }
    pub fn kind(&self) -> TargetKind {
        self.kind
    }
    pub fn objects(&self) -> &[TargetObject] {
        &self.objects
    }
}

/// Logical coverage before alpha, calibration, allocation or screening.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub c: f64,
    pub m: f64,
    pub y: f64,
    pub k: f64,
    pub w: f64,
}
const BLACK: Coverage = Coverage {
    c: 0.0,
    m: 0.0,
    y: 0.0,
    k: 1.0,
    w: 0.0,
};

/// Millimetres from top-left, x right and y down. Includes all strokes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub x_mm: f64,
    pub y_mm: f64,
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Shape {
    Rectangle,
    /// Text intent, not glyphs. A later renderer must declare its font and fit the box.
    Text {
        text: String,
    },
    /// Filled L: left and bottom arms, each 1/4 box thickness.
    AsymmetricMarker,
    /// Physical tick interval, starting at zero, never scaled with page size.
    Ruler {
        vertical: bool,
        tick_mm: f64,
    },
    /// Centered horizontal and vertical strokes, clipped within the box.
    RegistrationCross {
        stroke_mm: f64,
    },
    /// Alternating filled/clear stripes, starting filled at the box origin.
    LinePairs {
        vertical: bool,
        line_width_mm: f64,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetObject {
    pub label: String,
    pub bounds: Bounds,
    pub coverage: Coverage,
    /// Straight coverage is multiplied by alpha exactly once by a future renderer.
    pub alpha: f64,
    pub shape: Shape,
}
fn object(label: &str, bounds: [f64; 4], coverage: Coverage, shape: Shape) -> TargetObject {
    TargetObject {
        label: label.into(),
        bounds: Bounds {
            x_mm: bounds[0],
            y_mm: bounds[1],
            width_mm: bounds[2],
            height_mm: bounds[3],
        },
        coverage,
        alpha: 1.0,
        shape,
    }
}
pub fn generate_target(kind: TargetKind, size: PhysicalSize) -> DiagnosticTarget {
    let w = size.width_mm;
    let h = size.height_mm;
    let mut objects = vec![
        object(
            "Orientation text",
            [5.0, 5.0, w - 15.0, 3.0],
            BLACK,
            Shape::Text {
                text: "TOP LEFT R 123".into(),
            },
        ),
        object(
            "Off-center L marker",
            [w - 8.0, 5.0, 3.0, 5.0],
            BLACK,
            Shape::AsymmetricMarker,
        ),
        object(
            "Horizontal mm ruler",
            [5.0, h - 8.0, w - 15.0, 3.0],
            BLACK,
            Shape::Ruler {
                vertical: false,
                tick_mm: 1.0,
            },
        ),
        object(
            "Vertical mm ruler",
            [w - 8.0, 15.0, 3.0, h - 30.0],
            BLACK,
            Shape::Ruler {
                vertical: true,
                tick_mm: 1.0,
            },
        ),
    ];
    if kind == TargetKind::ChannelIdentity {
        let channels = [
            (
                "C",
                Coverage {
                    c: 1.0,
                    ..Coverage::default()
                },
            ),
            (
                "M",
                Coverage {
                    m: 1.0,
                    ..Coverage::default()
                },
            ),
            (
                "Y",
                Coverage {
                    y: 1.0,
                    ..Coverage::default()
                },
            ),
            ("K", BLACK),
            (
                "W",
                Coverage {
                    w: 1.0,
                    ..Coverage::default()
                },
            ),
            ("K + W overlap", Coverage { w: 1.0, ..BLACK }),
        ];
        let cell = (w - 20.0) / 3.0;
        for (i, (name, coverage)) in channels.into_iter().enumerate() {
            let x = 5.0 + (i % 3) as f64 * cell;
            let y = 15.0 + (i / 3) as f64 * 12.0;
            let label = if i < 5 {
                format!("Isolated {name}")
            } else {
                name.into()
            };
            objects.push(object(
                &label,
                [x, y, cell - 2.0, 6.0],
                coverage,
                Shape::Rectangle,
            ));
            objects.push(object(
                &format!("Label {name}"),
                [x, y + 7.0, cell - 2.0, 2.0],
                BLACK,
                Shape::Text { text: name.into() },
            ));
        }
    }
    match kind {
        TargetKind::ChannelIdentity => {}
        TargetKind::Geometry => {
            objects.push(object(
                "10 mm square",
                [10.0, 15.0, 10.0, 10.0],
                BLACK,
                Shape::Rectangle,
            ));
        }
        TargetKind::Registration => {
            for (label, x, y) in [
                ("Top left", 8.0, 15.0),
                ("Top right", w - 16.0, 15.0),
                ("Bottom left", 8.0, h - 16.0),
                ("Bottom right", w - 16.0, h - 16.0),
                ("Center", w / 2.0 - 3.0, h / 2.0 - 3.0),
            ] {
                objects.push(object(
                    label,
                    [x, y, 6.0, 6.0],
                    Coverage { w: 1.0, ..BLACK },
                    Shape::RegistrationCross { stroke_mm: 0.25 },
                ));
            }
        }
        TargetKind::WhiteRamp
        | TargetKind::WetLoad
        | TargetKind::AlphaEdge
        | TargetKind::Screening => {
            let cell = (w - 20.0) / 5.0;
            for (i, level) in [0.0, 0.25, 0.5, 0.75, 1.0].into_iter().enumerate() {
                let (name, coverage, alpha) = match kind {
                    TargetKind::WhiteRamp => (
                        "White",
                        Coverage {
                            w: level,
                            ..Coverage::default()
                        },
                        1.0,
                    ),
                    TargetKind::WetLoad => (
                        "Combined load",
                        Coverage {
                            c: level,
                            m: level,
                            y: level,
                            k: level,
                            w: level,
                        },
                        1.0,
                    ),
                    TargetKind::AlphaEdge => ("Alpha", Coverage { w: 1.0, ..BLACK }, level),
                    _ => (
                        "Screen field",
                        Coverage {
                            k: level,
                            ..Coverage::default()
                        },
                        1.0,
                    ),
                };
                let x = 5.0 + i as f64 * cell;
                let label = format!("{name} {}%", (level * 100.0) as u32);
                let mut patch = object(
                    &label,
                    [x, 15.0, cell - 1.0, 8.0],
                    coverage,
                    Shape::Rectangle,
                );
                patch.alpha = alpha;
                objects.push(patch);
                objects.push(object(
                    &format!("Label {label}"),
                    [x, 24.0, cell - 1.0, 2.0],
                    BLACK,
                    Shape::Text {
                        text: format!("{}%", (level * 100.0) as u32),
                    },
                ));
            }
            if kind == TargetKind::Screening {
                for (i, line_width_mm) in [0.1, 0.25, 0.5].into_iter().enumerate() {
                    objects.push(object(
                        &format!("Line pairs {line_width_mm} mm"),
                        [
                            5.0 + i as f64 * (w - 20.0) / 3.0,
                            29.0,
                            (w - 20.0) / 3.0 - 1.0,
                            8.0,
                        ],
                        BLACK,
                        Shape::LinePairs {
                            vertical: i != 1,
                            line_width_mm,
                        },
                    ));
                }
            }
        }
    }
    DiagnosticTarget {
        schema: "veylora.diagnostic-target".into(),
        version: 1,
        provenance: "synthetic geometry; not captured printer evidence or printer-ready output"
            .into(),
        kind,
        size,
        objects,
    }
}
