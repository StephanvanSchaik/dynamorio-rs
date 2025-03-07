#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "io")]
    Io(no_std_io::io::Error),
    /// The lock could not be acquired at this time because the operation would otherwise block.
    WouldBlock,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "io")]
            Error::Io(inner) => write!(f, "Error::Io({})", inner),
            Error::WouldBlock => write!(f, "Error::WouldBlock"),
        }
    }
}

#[cfg(feature = "io")]
impl From<no_std_io::io::Error> for Error {
    fn from(e: no_std_io::io::Error) -> Self {
        Self::Io(e)
    }
}
