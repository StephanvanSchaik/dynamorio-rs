use alloc::ffi::CString;
use dynamorio_sys::*;

use acid_io::Error;
use crate::io::{Read, Seek, SeekFrom, Write};
use crate::path::Path;

pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to `false`.
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be `read`-able if opened.
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be `write`-able if opened.
    ///
    /// If the file already exists, any write calls on it will overwrite its contents, without
    /// truncating it.
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// Sets the option for truncating a previous file.
    ///
    /// If a file is successfully opened with this option set it will truncate the file to 0 length
    /// if it already exists.
    ///
    /// The file must be opened with write access for truncate to work.
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    ///
    /// In order for the file to be created, [`OpenOptions::write`] or [`OpenOptions::append`]
    /// access must be used.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    ///
    /// No file is allowed to exist at the target location, also no (dangling) symbolic link. In
    /// this way, if the call succeeds, the file returned is guaranteed to be new.
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// Opens a file at `path` with the optons specified by `self`.
    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<File, Error> {
        let path = CString::new(path.as_ref().to_string_lossy().as_ref()).unwrap();
        let mut flags = 0;

        if self.read {
            flags |= DR_FILE_READ;
        }

        if self.write && self.append {
            flags |= DR_FILE_WRITE_APPEND;
        } else if self.write && self.truncate {
            flags |= DR_FILE_WRITE_OVERWRITE;
        } else if self.write && self.create_new {
            flags |= DR_FILE_WRITE_REQUIRE_NEW;
        }

        let inner = unsafe {
            dr_open_file(path.as_ptr(), flags)
        };

        if inner == INVALID_FILE {
            return Err(acid_io::Error::new(acid_io::ErrorKind::Other, "unknown"));
        }

        Ok(File {
            inner,
        })
    }
}

pub struct File {
    inner: file_t,
}

impl File {
    /// Attempts to open a file in read-only mode.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        File::options().read(true).open(path)
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file it it does not exist, and will truncate it if it does.
    ///
    /// See the [`OpenOptions::open`] function for more details.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        File::options().create(true).truncate(true).write(true).open(path)
    }

    /// Returns a new `OpenOptions` object.
    ///
    /// This function returns a new [`OpenOptions`] object that you can use to open or create a file
    /// with specific options if `open()` or `create()` are not appropriate.
    ///
    /// It is equivalent to `OpenOptions::new()` but allows you to write more readable code. Instead
    /// of `OpenOptions::new().read(true).open("foo.txt")` you can write
    /// `File::options().read(true).open("foo.txt")`. This also avoids the need to import
    /// `OpenOptions`.
    ///
    /// See the [`OpenOptions::new`] function for more details.
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        let inner = unsafe {
            dr_dup_file_handle(self.inner)
        };

        Ok(Self {
            inner,
        })
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, acid_io::Error> {
        let result = unsafe {
            dr_read_file(self.inner, buf.as_mut_ptr() as *mut core::ffi::c_void, buf.len())
        };

        if result < 0 {
            return Err(acid_io::Error::new(acid_io::ErrorKind::Other, "unknown"));
        }

        Ok(result as usize)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize, acid_io::Error> {
        let result = unsafe {
            dr_write_file(self.inner, buf.as_ptr() as *const core::ffi::c_void, buf.len())
        };

        if result < 0 {
            return Err(acid_io::Error::new(acid_io::ErrorKind::Other, "unknown"));
        }

        Ok(result as usize)
    }

    fn flush(&mut self) -> Result<(), acid_io::Error> {
        unsafe {
            dr_flush_file(self.inner);
        }

        Ok(())
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, acid_io::Error> {
        let (offset, origin) = match pos {
            SeekFrom::Start(offset)   => (offset as _, DR_SEEK_SET),
            SeekFrom::End(offset)     => (offset as _, DR_SEEK_END),
            SeekFrom::Current(offset) => (offset as _, DR_SEEK_CUR),
        };

        let result = unsafe {
            dr_file_seek(self.inner, offset, origin as i32) != 0
        };

        if !result {
            return Err(acid_io::Error::new(acid_io::ErrorKind::Other, "unknown"));
        }

        let result = unsafe {
            dr_file_tell(self.inner)
        };

        Ok(result as u64)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            dr_close_file(self.inner);
        }
    }
}
