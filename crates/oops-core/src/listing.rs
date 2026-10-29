//! Directory listing primitive used by the scanners.
//!
//! On macOS this uses `getattrlistbulk(2)`, which returns name, type, device
//! and allocated size for a whole batch of entries in one syscall -- the same
//! approach `fts(3)` (and therefore `du`) uses. A per-entry `lstat` on the full
//! path is several times slower and contends on kernel locks under rayon.
//! Elsewhere, or when following symlinks, it falls back to `read_dir` + stat.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::disk_size;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Dir,
    File,
    /// The entry exists but its attributes could not be read.
    Error,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Stat {
    pub kind: Kind,
    /// On-disk size (see `disk_size`). Zero for directories and errors.
    pub size: u64,
    pub dev: u64,
}

impl Stat {
    pub const ERROR: Stat = Stat {
        kind: Kind::Error,
        size: 0,
        dev: 0,
    };

    pub fn from_metadata(m: &fs::Metadata) -> Stat {
        if m.is_dir() {
            Stat {
                kind: Kind::Dir,
                size: 0,
                dev: m.dev(),
            }
        } else {
            Stat {
                kind: Kind::File,
                size: disk_size(m),
                dev: m.dev(),
            }
        }
    }

    pub fn of(path: &Path, follow_symlinks: bool) -> Stat {
        let meta = if follow_symlinks {
            fs::metadata(path)
        } else {
            fs::symlink_metadata(path)
        };
        meta.map_or(Stat::ERROR, |m| Stat::from_metadata(&m))
    }
}

/// Call `f` with the name and stat of every entry in `path` (excluding `.`/`..`).
///
/// `f` must not call `list_dir` itself: collect what you need and recurse after.
pub(crate) fn list_dir(
    path: &Path,
    follow_symlinks: bool,
    f: impl FnMut(&OsStr, Stat),
) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    if !follow_symlinks {
        return bulk::list_dir(path, f);
    }
    portable::list_dir(path, follow_symlinks, f)
}

mod portable {
    use super::*;

    pub(super) fn list_dir(
        path: &Path,
        follow_symlinks: bool,
        mut f: impl FnMut(&OsStr, Stat),
    ) -> io::Result<()> {
        for entry in fs::read_dir(path)?.flatten() {
            let stat = if follow_symlinks {
                Stat::of(&entry.path(), true)
            } else {
                entry
                    .metadata()
                    .map_or(Stat::ERROR, |m| Stat::from_metadata(&m))
            };
            f(&entry.file_name(), stat);
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod bulk {
    use super::*;

    use std::cell::RefCell;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::AsRawFd;

    // Not exported by the libc crate.
    const ATTR_CMN_ERROR: u32 = 0x2000_0000;
    const VDIR: u32 = 2;
    const BUF_LEN: usize = 128 * 1024;

    thread_local! {
        static BUF: RefCell<Vec<u8>> = RefCell::new(vec![0; BUF_LEN]);
    }

    pub(super) fn list_dir(path: &Path, mut f: impl FnMut(&OsStr, Stat)) -> io::Result<()> {
        let dir = fs::File::open(path)?;
        // SAFETY: attrlist is a plain C struct; all-zero is a valid value.
        let mut attrs: libc::attrlist = unsafe { std::mem::zeroed() };
        attrs.bitmapcount = libc::ATTR_BIT_MAP_COUNT;
        attrs.commonattr = libc::ATTR_CMN_RETURNED_ATTRS
            | ATTR_CMN_ERROR
            | libc::ATTR_CMN_NAME
            | libc::ATTR_CMN_DEVID
            | libc::ATTR_CMN_OBJTYPE;
        attrs.fileattr = libc::ATTR_FILE_TOTALSIZE | libc::ATTR_FILE_ALLOCSIZE;

        BUF.with_borrow_mut(|buf| loop {
            // SAFETY: `attrs` and `buf` are valid for the duration of the call
            // and `buf.len()` is the buffer's true size.
            let n = unsafe {
                libc::getattrlistbulk(
                    dir.as_raw_fd(),
                    (&mut attrs as *mut libc::attrlist).cast(),
                    buf.as_mut_ptr().cast(),
                    buf.len(),
                    libc::FSOPT_PACK_INVAL_ATTRS as u64,
                )
            };
            if n < 0 {
                return Err(io::Error::last_os_error());
            }
            if n == 0 {
                return Ok(());
            }
            let mut off = 0;
            for _ in 0..n {
                off += parse_entry(&buf[off..], &mut f);
            }
        })
    }

    /// Parse one packed record and return its length. With
    /// `FSOPT_PACK_INVAL_ATTRS` every requested attribute is present, so the
    /// layout is fixed (attributes in bitmap order, after the returned set):
    ///
    /// ```text
    /// u32 length | attribute_set_t returned (5 x u32) | u32 error
    /// | attrreference_t name | dev_t devid | u32 objtype
    /// | off_t totalsize | off_t allocsize
    /// ```
    fn parse_entry(rec: &[u8], f: &mut impl FnMut(&OsStr, Stat)) -> usize {
        let len = u32_at(rec, 0) as usize;
        let error = u32_at(rec, 24);
        let name_ref = 28;
        let name_off = u32_at(rec, name_ref) as i32;
        let name_len = u32_at(rec, name_ref + 4) as usize;
        let dev = u32_at(rec, 36) as i32;
        let objtype = u32_at(rec, 40);
        let total = u64_at(rec, 44);
        let alloc = u64_at(rec, 52);

        // name_off is relative to the attrreference_t; name_len includes the NUL.
        let start = (name_ref as isize + name_off as isize) as usize;
        let name = OsStr::from_bytes(&rec[start..start + name_len.saturating_sub(1)]);

        let stat = if error != 0 {
            Stat::ERROR
        } else if objtype == VDIR {
            Stat {
                kind: Kind::Dir,
                size: 0,
                // Matches MetadataExt::dev(), which sign-extends st_dev.
                dev: dev as u64,
            }
        } else {
            Stat {
                kind: Kind::File,
                size: if alloc > 0 { alloc } else { total },
                dev: dev as u64,
            }
        };
        f(name, stat);
        len
    }

    fn u32_at(b: &[u8], at: usize) -> u32 {
        u32::from_ne_bytes(b[at..at + 4].try_into().unwrap())
    }

    fn u64_at(b: &[u8], at: usize) -> u64 {
        u64::from_ne_bytes(b[at..at + 8].try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn collect(path: &Path, bulk: bool) -> BTreeMap<String, (Kind, u64, u64)> {
        let mut out = BTreeMap::new();
        let mut add = |name: &OsStr, s: Stat| {
            out.insert(name.to_string_lossy().into_owned(), (s.kind, s.size, s.dev));
        };
        if bulk {
            list_dir(path, false, &mut add).unwrap();
        } else {
            portable::list_dir(path, false, &mut add).unwrap();
        }
        out
    }

    #[test]
    fn bulk_listing_matches_stat() {
        let dir = std::env::temp_dir().join(format!("oops-listing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("small"), b"hi").unwrap();
        fs::write(dir.join("big"), vec![7u8; 300_000]).unwrap();
        fs::write(dir.join("empty"), b"").unwrap();
        fs::write(dir.join("ünïcödé name"), b"x").unwrap();
        std::os::unix::fs::symlink("small", dir.join("link")).unwrap();

        let bulk = collect(&dir, true);
        let portable = collect(&dir, false);
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(bulk.len(), 6);
        assert_eq!(bulk, portable);
        assert_eq!(bulk["sub"].0, Kind::Dir);
    }

    #[test]
    fn bulk_listing_matches_stat_on_large_dir() {
        // Spans multiple getattrlistbulk batches on macOS.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dir = dir.join("target");
        if !dir.exists() {
            return;
        }
        for sub in ["debug/deps", "release/deps"] {
            let p = dir.join(sub);
            if p.exists() {
                assert_eq!(collect(&p, true), collect(&p, false));
            }
        }
    }
}
