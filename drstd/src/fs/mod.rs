//! Filesystem manipulation operations.
//!
//! This module contains basic method to manipulate the contents of the local filesystem. All
//! methods in this module represent cross-platform filesystem operations.

mod file;

use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use dynamorio_sys::*;

pub use file::File;

use crate::io::Read;
use crate::path::Path;

/// Creates a new, empty directory at the provided path.
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<(), no_std_io::io::Error> {
    let path = CString::new(path.as_ref().to_string_lossy().as_ref()).unwrap();

    let result = unsafe {
        dr_create_dir(path.as_ptr()) != 0
    };

    if !result {
        return Err(no_std_io::io::Error::new(no_std_io::io::ErrorKind::Other, "unknown"));
    }

    Ok(())
}

/// Reads the entire contents of a file into a bytes vector.
///
/// This is a convenience function for using [`File::open`] and [`read_to_end`] with fewer imports
/// and without an intermediate variable.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, no_std_io::io::Error> {
    let mut file = File::open(path).unwrap();
    let mut buf = vec![];

    file.read_to_end(&mut buf)?;

    Ok(buf)
}

/// Read the entire contents of a file into a string.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, no_std_io::io::Error> {
    let buf = read(path)?;

    Ok(String::from_utf8(buf).unwrap())
}

/// Removes an empty directory.
pub fn remove_dir<P: AsRef<Path>>(path: P) -> Result<(), no_std_io::io::Error> {
    let path = CString::new(path.as_ref().to_string_lossy().as_ref()).unwrap();

    let result = unsafe {
        dr_delete_dir(path.as_ptr()) != 0
    };

    if !result {
        return Err(no_std_io::io::Error::new(no_std_io::io::ErrorKind::Other, "unknown"));
    }

    Ok(())
}

/// Removes a file from the filesystem.
///
/// Note that there is no guarantee that the file is immediately deleted (e.g., depending on
/// platform, other open file descriptors may prevent immediate removal.
pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), no_std_io::io::Error> {
    let path = CString::new(path.as_ref().to_string_lossy().as_ref()).unwrap();

    let result = unsafe {
        dr_delete_file(path.as_ptr()) != 0
    };

    if !result {
        return Err(no_std_io::io::Error::new(no_std_io::io::ErrorKind::Other, "unknown"));
    }

    Ok(())
}

/// Rename a file or directory to a new name, replacing the original file if `to` already exists.
///
/// This will not work if the new name is on a different mount point.
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), no_std_io::io::Error> {
    let from = CString::new(from.as_ref().to_string_lossy().as_ref()).unwrap();
    let to = CString::new(to.as_ref().to_string_lossy().as_ref()).unwrap();

    let result = unsafe {
        dr_rename_file(from.as_ptr(), to.as_ptr(), true as i8) != 0
    };

    if !result {
        return Err(no_std_io::io::Error::new(no_std_io::io::ErrorKind::Other, "unknown"));
    }

    Ok(())
}
