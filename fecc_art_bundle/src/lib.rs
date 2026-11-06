// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Bundles the `art` PNGs for the web build, one bundle per part type.
//!
//! A thousand small requests cost more than the art itself, so the web build fetches each
//! part type's files in one go. A bundle holds the original PNG files back to back:
//!
//! - [`MAGIC`]
//! - then for each file: name length (`u16` LE), UTF-8 name, data length (`u32` LE), data

use std::error::Error;
use std::fmt::{self, Display};
use std::ops::Range;

/// The bytes a bundle starts with.
pub const MAGIC: &[u8; 8] = b"FECCART1";

/// Returns the bundle an `art` file belongs in: its part type, such as `Face` for `Lyn_Face.png`.
///
/// Only plain ASCII types count, as the type is part of the bundle's URL.
pub fn bundle_key(file_name: &str) -> Option<&str> {
    let (_, key) = file_name.strip_suffix(".png")?.rsplit_once('_')?;
    (!key.is_empty() && key.bytes().all(|byte| byte.is_ascii_alphanumeric())).then_some(key)
}

/// Why a bundle couldn't be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidBundle {
    /// It doesn't start with [`MAGIC`]
    NotABundle,
    /// It ends partway through a file
    Truncated,
    /// A file name isn't UTF-8
    BadName,
}

impl Display for InvalidBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotABundle => "not an art bundle",
            Self::Truncated => "the art bundle is cut short",
            Self::BadName => "the art bundle has a file name that isn't UTF-8",
        })
    }
}

impl Error for InvalidBundle {}

/// Lists the files in a bundle, with where each file's data sits in it.
pub fn entries(bundle: &[u8]) -> Result<Vec<(&str, Range<usize>)>, InvalidBundle> {
    let mut rest = bundle
        .strip_prefix(MAGIC.as_slice())
        .ok_or(InvalidBundle::NotABundle)?;
    let mut entries = Vec::new();

    while !rest.is_empty() {
        let name_len = u16::from_le_bytes(take(&mut rest)?);
        let name = take_slice(&mut rest, usize::from(name_len))?;
        let name = std::str::from_utf8(name)
            .ok()
            .ok_or(InvalidBundle::BadName)?;

        let data_len = usize::try_from(u32::from_le_bytes(take(&mut rest)?))
            .ok()
            .ok_or(InvalidBundle::Truncated)?;
        let start = bundle.len() - rest.len();
        take_slice(&mut rest, data_len)?;

        entries.push((name, start..start + data_len));
    }

    Ok(entries)
}

/// Takes a fixed number of bytes from the front.
fn take<const N: usize>(rest: &mut &[u8]) -> Result<[u8; N], InvalidBundle> {
    let (bytes, after) = rest.split_first_chunk().ok_or(InvalidBundle::Truncated)?;
    *rest = after;
    Ok(*bytes)
}

/// Takes `len` bytes from the front.
fn take_slice<'a>(rest: &mut &'a [u8], len: usize) -> Result<&'a [u8], InvalidBundle> {
    let (bytes, after) = rest.split_at_checked(len).ok_or(InvalidBundle::Truncated)?;
    *rest = after;
    Ok(bytes)
}

/// Builds a bundle from file names and their data.
///
/// # Panics
///
/// If a name is longer than 65,535 bytes, or a file is 4 GiB or more.
pub fn write_bundle<'a>(files: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> Vec<u8> {
    let mut bundle = MAGIC.to_vec();
    for (name, data) in files {
        let name_len = u16::try_from(name.len()).expect("Art file names fit in a u16");
        let data_len = u32::try_from(data.len()).expect("Art files fit in a u32");
        bundle.extend_from_slice(&name_len.to_le_bytes());
        bundle.extend_from_slice(name.as_bytes());
        bundle.extend_from_slice(&data_len.to_le_bytes());
        bundle.extend_from_slice(data);
    }
    bundle
}

#[cfg(feature = "write")]
pub use write::{Bundle, art_files, bundles, gzip};

#[cfg(feature = "write")]
mod write {
    use super::{bundle_key, write_bundle};
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use sha2::{Digest as _, Sha256};
    use std::collections::BTreeMap;
    use std::io::Write as _;
    use std::path::Path;
    use std::{fs, io};

    /// Compresses a bundle at the highest level, as it's done once per build.
    ///
    /// Bundles are served compressed, as Cloudflare doesn't compress binary files, and the
    /// browser decompresses them natively.
    pub fn gzip(contents: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(contents)
            .expect("Writing to a Vec can't fail");
        encoder.finish().expect("Writing to a Vec can't fail")
    }

    /// One part type's bundle, before compression.
    pub struct Bundle {
        /// The part type, such as `Face`
        pub key: String,
        /// The file name, with a hash of the contents so it can be cached forever
        pub file_name: String,
        /// The bundle itself
        pub contents: Vec<u8>,
    }

    /// Lists the PNG files in an `art` folder, sorted, as the order is the display order.
    pub fn art_files(art_dir: &Path) -> io::Result<Vec<String>> {
        let mut file_names = Vec::new();
        for entry in fs::read_dir(art_dir)? {
            let path = entry?.path();
            if path.is_file()
                && path.extension().is_some_and(|extension| extension == "png")
                && let Some(file_name) = path.file_name().and_then(|name| name.to_str())
            {
                file_names.push(file_name.to_owned());
            }
        }
        file_names.sort();
        Ok(file_names)
    }

    /// Builds a bundle for each part type in an `art` folder, sorted by type.
    ///
    /// Files with no part type are left out, as the app can't use them.
    pub fn bundles(art_dir: &Path) -> io::Result<Vec<Bundle>> {
        let mut by_key: BTreeMap<String, Vec<(String, Vec<u8>)>> = BTreeMap::new();
        for file_name in art_files(art_dir)? {
            if let Some(key) = bundle_key(&file_name) {
                let data = fs::read(art_dir.join(&file_name))?;
                by_key
                    .entry(key.to_owned())
                    .or_default()
                    .push((file_name, data));
            }
        }

        Ok(by_key
            .into_iter()
            .map(|(key, files)| {
                let contents = write_bundle(
                    files
                        .iter()
                        .map(|(name, data)| (name.as_str(), data.as_slice())),
                );
                let file_name = format!("{key}-{}.gz", short_hash(&contents));
                Bundle {
                    key,
                    file_name,
                    contents,
                }
            })
            .collect())
    }

    /// The first 8 bytes of the SHA-256 hash, in hex.
    fn short_hash(contents: &[u8]) -> String {
        Sha256::digest(contents)[..8]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_key_is_the_part_type() {
        assert_eq!(bundle_key("Lyn_Face.png"), Some("Face"), "Plain name");
        assert_eq!(
            bundle_key("Lyn[FE7]{Iscaneus}_HairBack.png"),
            Some("HairBack"),
            "Tags come before the type"
        );
        assert_eq!(bundle_key("Lyn_Face.jpg"), None, "Not a PNG");
        assert_eq!(bundle_key("Lyn.png"), None, "No type");
        assert_eq!(bundle_key("Lyn_.png"), None, "Empty type");
        assert_eq!(bundle_key("Lyn_Fa ce.png"), None, "Not URL-safe");
    }

    #[test]
    fn entries_reads_back_what_write_bundle_wrote() {
        let files: [(&str, &[u8]); 3] = [
            ("A_Face.png", b"first"),
            ("B_Face.png", b""),
            ("Ç_Face.png", b"third file"),
        ];
        let bundle = write_bundle(files);

        let read: Vec<(&str, &[u8])> = entries(&bundle)
            .expect("A written bundle should be readable")
            .into_iter()
            .map(|(name, range)| (name, &bundle[range]))
            .collect();
        assert_eq!(
            read, files,
            "Should list every file, in order, with its data"
        );
    }

    #[test]
    fn entries_of_an_empty_bundle_is_empty() {
        let bundle = write_bundle([]);
        assert_eq!(entries(&bundle), Ok(Vec::new()), "Nothing in, nothing out");
    }

    #[test]
    fn entries_rejects_bad_bundles() {
        assert_eq!(
            entries(b"PNG...").map(|_| ()),
            Err(InvalidBundle::NotABundle),
            "Wrong magic"
        );

        let bundle = write_bundle([("A_Face.png", b"data".as_slice())]);
        for len in MAGIC.len() + 1..bundle.len() {
            assert_eq!(
                entries(&bundle[..len]).map(|_| ()),
                Err(InvalidBundle::Truncated),
                "Cut short at {len} bytes"
            );
        }

        let mut bad_name = bundle;
        bad_name[MAGIC.len() + 2] = 0xff;
        assert_eq!(
            entries(&bad_name).map(|_| ()),
            Err(InvalidBundle::BadName),
            "Name isn't UTF-8"
        );
    }
}
