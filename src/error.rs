use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrengineError {
    #[error("Generic error: {0}")]
    Generic(String),

    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Tobj model loading error")]
    Tobj(#[from] tobj::LoadError),
    
    #[error("Image loading error")]
    Image(#[from] image::ImageError),

    #[error("Failed to request a wgpu device")]
    WgpuRequestDevice(#[from] wgpu::RequestDeviceError),

    #[error("Failed to create a wgpu surface")]
    WgpuCreateSurface(#[from] wgpu::CreateSurfaceError),

    #[error("No suitable GPU adapter found")]
    NoGpuAdapter,

    #[error("Mismatched material count in model")]
    MismatchedMaterials,
}

pub type Result<T> = std::result::Result<T, OrengineError>;
