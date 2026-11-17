use veylora_domain::preflight::Finding;
use veylora_image::ImageError;
use veylora_persistence::PersistenceError;
use veylora_render::RenderError;

#[derive(Debug)]
pub enum WorkflowError {
    Persistence(PersistenceError),
    Image(ImageError),
    Render(RenderError),
    /// The request is inconsistent with itself or with stored records.
    Invalid(String),
    /// Preflight found blocking findings. Nothing was rendered.
    PreflightBlocked(Vec<Finding>),
    /// The printer may have received data but the outcome could not be saved. The job stays open
    /// until `recover` marks it for operator review. It is never retried.
    OutcomeNotRecorded(String),
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Persistence(e) => write!(f, "{e}"),
            Self::Image(e) => write!(f, "{e}"),
            Self::Render(e) => write!(f, "render failed: {e:?}"),
            Self::Invalid(m) => f.write_str(m),
            Self::PreflightBlocked(findings) => {
                write!(f, "preflight blocked the render:")?;
                for x in findings {
                    write!(f, " [{}] {}", x.code, x.message)?;
                }
                Ok(())
            }
            Self::OutcomeNotRecorded(m) => write!(f, "submission outcome not recorded: {m}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for WorkflowError {}

macro_rules! from {
    ($($ty:ty => $variant:ident),*) => {$(
        impl From<$ty> for WorkflowError {
            fn from(e: $ty) -> Self { Self::$variant(e) }
        }
    )*};
}
from!(PersistenceError => Persistence, ImageError => Image, RenderError => Render,
    std::io::Error => Io, serde_json::Error => Json);

pub type WorkflowResult<T> = Result<T, WorkflowError>;
