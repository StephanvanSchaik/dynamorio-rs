#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "io")]
    AcidIo(acid_io::Error),
    /// The lock could not be acquired at this time because the operation would otherwise block.
    WouldBlock,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "io")]
            Error::AcidIo(inner) => write!(f, "Error::AcidIo({})", inner),
            Error::WouldBlock => write!(f, "Error::WouldBlock"),
        }
    }
}

#[cfg(feature = "io")]
impl From<acid_io::Error> for Error {
    fn from(e: acid_io::Error) -> Self {
        Self::AcidIo(e)
    }
}
