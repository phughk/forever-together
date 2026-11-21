//! The install's secret key as a standard OpenSSH ed25519 private key file (ADR 0039).
//!
//! The key itself stays 32 bytes in the store (`identity/secret`). These functions read and
//! write it in the format that `ssh-keygen`, `age` and similar tools understand: unencrypted, no
//! passphrase (decision 1). They return data and never print; the command line layer
//! (`runtime::run_key_command`) prints, prompts and takes the data directory lock.
//!
//! Generic over the store, with no `dyn` (decision 7 of ticket 0043's design).

use crate::identity::{Identity, IdentityError, PeerId, Signer};
use crate::store::{KEY_IDENTITY_SECRET, Store, StoreError};
use data_encoding::BASE64;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAGIC: &[u8] = b"openssh-key-v1\0";
const KEY_TYPE: &[u8] = b"ssh-ed25519";
const BEGIN: &str = "-----BEGIN OPENSSH PRIVATE KEY-----";
const END: &str = "-----END OPENSSH PRIVATE KEY-----";
/// A key file is a few hundred bytes. Refuse to read anything larger.
const MAX_FILE_BYTES: u64 = 16 * 1024;

/// What the store holds before a key is replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OldKey {
    /// No key was stored.
    None,
    Id(PeerId),
    /// A value is stored but it is not a 32-byte secret.
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInfo {
    pub peer_id: PeerId,
    /// `ssh-ed25519 <base64>`, the line `ssh-keygen -y` prints.
    pub openssh_public: String,
}

/// The result of replacing the stored key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Replaced {
    pub old: OldKey,
    pub new: PeerId,
}

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("{0} already exists; it is not overwritten")]
    FileExists(PathBuf),
    #[error("key file: {0}")]
    Io(String),
    #[error("not an OpenSSH private key file")]
    NotOpenSsh,
    #[error("the key file is protected by a passphrase; only unencrypted keys are supported")]
    Encrypted,
    #[error("the key file holds a {0} key; only ed25519 is supported")]
    WrongKeyType(String),
    #[error("the key file is damaged: {0}")]
    BadKeyFile(&'static str),
    #[error("the stored key is damaged; replace it with `key reset` or `key import --force`")]
    BadStoredKey,
    #[error("a key is already stored; pass --force to replace it")]
    WouldReplace(Replaced),
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl From<IdentityError> for KeyError {
    fn from(e: IdentityError) -> Self {
        match e {
            IdentityError::Store(s) => KeyError::Store(s),
            IdentityError::BadStoredKey => KeyError::BadStoredKey,
        }
    }
}

/// What the store holds now, without creating anything.
pub fn old_key<S: Store + ?Sized>(store: &S) -> Result<OldKey, KeyError> {
    Ok(match store.get(KEY_IDENTITY_SECRET)? {
        None => OldKey::None,
        Some(b) => match <[u8; 32]>::try_from(b) {
            Ok(secret) => OldKey::Id(Identity::from_secret(secret).peer_id()),
            Err(_) => OldKey::Unreadable,
        },
    })
}

/// The stored key's peer ID and public line. Creates a key if none is stored (decision 8).
pub fn show<S: Store + ?Sized>(store: &S) -> Result<KeyInfo, KeyError> {
    let id = Identity::load_or_create(store)?;
    Ok(KeyInfo {
        peer_id: id.peer_id(),
        openssh_public: openssh_public(&id.peer_id().0),
    })
}

/// The stored key as OpenSSH private key text. Creates a key if none is stored.
pub fn export_text<S: Store + ?Sized>(store: &S) -> Result<(PeerId, String), KeyError> {
    let id = Identity::load_or_create(store)?;
    Ok((id.peer_id(), encode_private(&id.secret_bytes())))
}

/// Write the stored key to a new file, owner-only on Unix, in one create-exclusive open. An
/// existing file is never touched. A partly written file is removed.
pub fn export_to_file<S: Store + ?Sized>(store: &S, path: &Path) -> Result<PeerId, KeyError> {
    let (peer, text) = export_text(store)?;
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let mut f = o.open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            KeyError::FileExists(path.to_path_buf())
        } else {
            KeyError::Io(format!("{}: {e}", path.display()))
        }
    })?;
    if let Err(e) = f.write_all(text.as_bytes()).and_then(|()| f.sync_all()) {
        drop(f);
        let _ = std::fs::remove_file(path);
        return Err(KeyError::Io(format!("{}: {e}", path.display())));
    }
    Ok(peer)
}

/// Replace the stored key with the one in `path`. The file is read and fully checked before the
/// store is touched, and the store is written once. A stored key (even a damaged one) is replaced
/// only with `force`.
pub fn import_from_file<S: Store + ?Sized>(
    store: &S,
    path: &Path,
    force: bool,
) -> Result<Replaced, KeyError> {
    let io = |e: std::io::Error| KeyError::Io(format!("{}: {e}", path.display()));
    let meta = std::fs::metadata(path).map_err(io)?;
    if meta.len() > MAX_FILE_BYTES {
        return Err(KeyError::NotOpenSsh);
    }
    let text = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            KeyError::NotOpenSsh
        } else {
            io(e)
        }
    })?;
    let secret = decode_private(&text)?;
    let new = Identity::from_secret(secret).peer_id();
    let old = old_key(store)?;
    if old != OldKey::None && !force {
        return Err(KeyError::WouldReplace(Replaced { old, new }));
    }
    store.put(KEY_IDENTITY_SECRET, &secret)?;
    Ok(Replaced { old, new })
}

/// Discard the stored key and store a new random one, in one write.
pub fn reset<S: Store + ?Sized>(store: &S) -> Result<Replaced, KeyError> {
    let old = old_key(store)?;
    let id = Identity::generate();
    store.put(KEY_IDENTITY_SECRET, &id.secret_bytes())?;
    Ok(Replaced {
        old,
        new: id.peer_id(),
    })
}

// ---- OpenSSH encoding -------------------------------------------------------------------

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn put_str(out: &mut Vec<u8>, s: &[u8]) {
    put_u32(out, s.len() as u32);
    out.extend_from_slice(s);
}

fn public_blob(public: &[u8; 32]) -> Vec<u8> {
    let mut b = Vec::new();
    put_str(&mut b, KEY_TYPE);
    put_str(&mut b, public);
    b
}

fn openssh_public(public: &[u8; 32]) -> String {
    format!("ssh-ed25519 {}", BASE64.encode(&public_blob(public)))
}

fn encode_private(secret: &[u8; 32]) -> String {
    let public = Identity::from_secret(*secret).peer_id().0;
    let mut check = [0u8; 4];
    rand::fill(&mut check);
    let check = u32::from_be_bytes(check);
    let mut private = Vec::new();
    put_u32(&mut private, check);
    put_u32(&mut private, check);
    put_str(&mut private, KEY_TYPE);
    put_str(&mut private, &public);
    let mut pair = secret.to_vec();
    pair.extend_from_slice(&public);
    put_str(&mut private, &pair);
    put_str(&mut private, b""); // comment
    let mut pad = 1u8;
    while private.len() % 8 != 0 {
        private.push(pad);
        pad += 1;
    }
    let mut body = Vec::new();
    body.extend_from_slice(MAGIC);
    put_str(&mut body, b"none");
    put_str(&mut body, b"none");
    put_str(&mut body, b"");
    put_u32(&mut body, 1);
    put_str(&mut body, &public_blob(&public));
    put_str(&mut body, &private);
    let b64 = BASE64.encode(&body);
    let mut out = String::from(BEGIN);
    out.push('\n');
    for line in b64.as_bytes().chunks(70) {
        out.push_str(std::str::from_utf8(line).expect("base64 is ASCII"));
        out.push('\n');
    }
    out.push_str(END);
    out.push('\n');
    out
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], KeyError> {
        if self.0.len() < n {
            return Err(KeyError::BadKeyFile("it ends early"));
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }
    fn u32(&mut self) -> Result<u32, KeyError> {
        Ok(u32::from_be_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }
    fn string(&mut self) -> Result<&'a [u8], KeyError> {
        let n = self.u32()? as usize;
        self.take(n)
    }
}

fn decode_private(text: &str) -> Result<[u8; 32], KeyError> {
    let text = text.trim();
    let body = text.strip_prefix(BEGIN).ok_or(KeyError::NotOpenSsh)?;
    let end = body.find(END).ok_or(KeyError::NotOpenSsh)?;
    if !body[end + END.len()..].trim().is_empty() {
        return Err(KeyError::BadKeyFile("there is data after the end marker"));
    }
    let b64: String = body[..end].split_whitespace().collect();
    let bytes = BASE64
        .decode(b64.as_bytes())
        .map_err(|_| KeyError::BadKeyFile("the key data is not valid base64"))?;
    let mut r = Reader(&bytes);
    if r.take(MAGIC.len())? != MAGIC {
        return Err(KeyError::NotOpenSsh);
    }
    let cipher = r.string()?;
    let kdf = r.string()?;
    let _kdf_options = r.string()?;
    if cipher != b"none" || kdf != b"none" {
        return Err(KeyError::Encrypted);
    }
    if r.u32()? != 1 {
        return Err(KeyError::BadKeyFile("it does not hold exactly one key"));
    }
    let blob = r.string()?;
    let mut b = Reader(blob);
    let blob_type = b.string()?;
    if blob_type != KEY_TYPE {
        return Err(KeyError::WrongKeyType(
            String::from_utf8_lossy(blob_type).into_owned(),
        ));
    }
    let public: [u8; 32] = b
        .string()?
        .try_into()
        .map_err(|_| KeyError::BadKeyFile("the public key is not 32 bytes"))?;
    let section = r.string()?;
    if !r.0.is_empty() {
        return Err(KeyError::BadKeyFile("there is data after the key"));
    }
    let mut p = Reader(section);
    if p.u32()? != p.u32()? {
        return Err(KeyError::BadKeyFile("the check numbers differ"));
    }
    if p.string()? != KEY_TYPE {
        return Err(KeyError::BadKeyFile("the key types differ"));
    }
    let public_again = p.string()?;
    let pair: [u8; 64] = p
        .string()?
        .try_into()
        .map_err(|_| KeyError::BadKeyFile("the private key is not 64 bytes"))?;
    let _comment = p.string()?;
    for (i, pad) in p.0.iter().enumerate() {
        if *pad as usize != i + 1 {
            return Err(KeyError::BadKeyFile("the padding is wrong"));
        }
    }
    let secret: [u8; 32] = pair[..32].try_into().expect("32 bytes");
    let derived = Identity::from_secret(secret).peer_id().0;
    if derived != public || public_again != public || pair[32..] != public {
        return Err(KeyError::BadKeyFile("the public and private halves differ"));
    }
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemStore;

    // Throwaway keys made with `ssh-keygen` for these tests only (no passphrase unless named).
    const ED: &str = r#"-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBHwMIplDAQGYhA3Bf0jRTcMt14KdUxnMEu6bk1NNdsdAAAAIjw6UOB8OlD
gQAAAAtzc2gtZWQyNTUxOQAAACBHwMIplDAQGYhA3Bf0jRTcMt14KdUxnMEu6bk1NNdsdA
AAAEBzZ7eK03OUZyerh0VhYrK0RqbHCg9xEa0Tcg0qTdRT8kfAwimUMBAZiEDcF/SNFNwy
3Xgp1TGcwS7puTU012x0AAAAAAECAwQF
-----END OPENSSH PRIVATE KEY-----
"#;
    const ED_PUBLIC: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEfAwimUMBAZiEDcF/SNFNwy3Xgp1TGcwS7puTU012x0";
    const ED_ENCRYPTED: &str = r#"-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABDEhiOf+v
vOoNWQgAjzGP2kAAAAGAAAAAEAAAAzAAAAC3NzaC1lZDI1NTE5AAAAINSmEdGDCwVfYrRw
2d3j4Xqbiu4+I61aeCHPhHXZHa9iAAAAkKwcOl3qcudFVnhWV5Jil18j8q4XIB0OHtnF+2
svBOrOz/rHR8rthdTek2WNRmy3Iklm15M7zXs6LMEryomX9lQgsvVSzAywiHFIzUpsSIt4
+hgzfYxZz/6Sawg1MEg99zOpoCJJpje5B8/QY/DXSkM4leIecoIYqMUKX/5Fd07hH6N2xx
M3kQKPOlP5YTy69A==
-----END OPENSSH PRIVATE KEY-----
"#;
    const ECDSA: &str = r#"-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAaAAAABNlY2RzYS
1zaGEyLW5pc3RwMjU2AAAACG5pc3RwMjU2AAAAQQQa4QEm6dATx+2jMfGMo4KwwF6gmOxT
cDGSwOmMKwLgQbS+hahLnCn0WvBMfCVF+tajTTR83u6FL+vpF+OvfLupAAAAoCLR5zci0e
c3AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBBrhASbp0BPH7aMx
8YyjgrDAXqCY7FNwMZLA6YwrAuBBtL6FqEucKfRa8Ex8JUX61qNNNHze7oUv6+kX4698u6
kAAAAhAN3fiYLlxGwIn/1BsZvW8NWE9kjI/wEYUXoOax8ZRCg1AAAAAAECAwQFBgc=
-----END OPENSSH PRIVATE KEY-----
"#;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wowft-keys-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn file(dir: &Path, name: &str, text: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, text).unwrap();
        p
    }

    fn stored(s: &MemStore) -> Option<Vec<u8>> {
        s.get(KEY_IDENTITY_SECRET).unwrap()
    }

    #[test]
    fn an_ssh_keygen_key_is_read_and_its_public_line_matches() {
        let d = tmp("vector");
        let s = MemStore::new();
        let r = import_from_file(&s, &file(&d, "k", ED), false).unwrap();
        assert_eq!(r.old, OldKey::None);
        let info = show(&s).unwrap();
        assert_eq!(info.peer_id, r.new);
        // `ssh-keygen -y -f` printed this line for the same file.
        assert_eq!(info.openssh_public, ED_PUBLIC);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn export_then_import_gives_the_same_peer_id() {
        let d = tmp("roundtrip");
        let a = MemStore::new();
        let id = show(&a).unwrap().peer_id;
        let peer = export_to_file(&a, &d.join("k")).unwrap();
        assert_eq!(peer, id);
        let b = MemStore::new();
        let r = import_from_file(&b, &d.join("k"), false).unwrap();
        assert_eq!(r.new, id);
        assert_eq!(show(&b).unwrap().peer_id, id);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn the_exported_text_is_what_the_file_holds_and_has_the_openssh_shape() {
        let d = tmp("text");
        let s = MemStore::new();
        let (_, text) = export_text(&s).unwrap();
        export_to_file(&s, &d.join("k")).unwrap();
        // Fresh random check numbers differ per call, so compare the shape and the key.
        let on_disk = std::fs::read_to_string(d.join("k")).unwrap();
        assert!(on_disk.starts_with(BEGIN) && on_disk.trim_end().ends_with(END));
        assert!(
            text.lines()
                .all(|l| l.len() <= 70 || l.starts_with("-----"))
        );
        assert_eq!(
            decode_private(&on_disk).unwrap(),
            decode_private(&text).unwrap()
        );
        let _ = std::fs::remove_dir_all(d);
    }

    #[cfg(unix)]
    #[test]
    fn an_exported_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let d = tmp("mode");
        export_to_file(&MemStore::new(), &d.join("k")).unwrap();
        let m = std::fs::metadata(d.join("k")).unwrap().permissions().mode();
        assert_eq!(m & 0o777, 0o600);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn export_never_overwrites_an_existing_file() {
        let d = tmp("exists");
        let p = file(&d, "k", "keep me");
        let r = export_to_file(&MemStore::new(), &p);
        assert!(matches!(r, Err(KeyError::FileExists(_))));
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "keep me");
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn export_into_a_missing_directory_is_an_io_error() {
        let d = tmp("nodir");
        let r = export_to_file(&MemStore::new(), &d.join("missing/k"));
        assert!(matches!(r, Err(KeyError::Io(_))));
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn show_and_export_create_a_key_on_an_empty_store_and_keep_it() {
        let s = MemStore::new();
        assert_eq!(stored(&s), None);
        let a = show(&s).unwrap().peer_id;
        assert_eq!(stored(&s).unwrap().len(), 32);
        assert_eq!(show(&s).unwrap().peer_id, a);
        let s2 = MemStore::new();
        let (peer, _) = export_text(&s2).unwrap();
        assert_eq!(stored(&s2).unwrap().len(), 32);
        assert_eq!(show(&s2).unwrap().peer_id, peer);
    }

    #[test]
    fn a_key_made_by_show_needs_force_to_be_replaced() {
        let d = tmp("force");
        let s = MemStore::new();
        let first = show(&s).unwrap().peer_id;
        let p = file(&d, "k", ED);
        let r = import_from_file(&s, &p, false);
        match r {
            Err(KeyError::WouldReplace(rep)) => {
                assert_eq!(rep.old, OldKey::Id(first));
                assert_ne!(rep.new, first);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(show(&s).unwrap().peer_id, first, "store unchanged");
        let r = import_from_file(&s, &p, true).unwrap();
        assert_eq!(r.old, OldKey::Id(first));
        assert_eq!(show(&s).unwrap().peer_id, r.new);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_failed_import_leaves_the_store_unchanged() {
        let d = tmp("unchanged");
        let s = MemStore::new();
        let before = show(&s).unwrap().peer_id;
        for (name, text) in [
            ("garbage", "hello"),
            ("ecdsa", ECDSA),
            ("encrypted", ED_ENCRYPTED),
        ] {
            let r = import_from_file(&s, &file(&d, name, text), true);
            assert!(r.is_err(), "{name}");
            assert_eq!(show(&s).unwrap().peer_id, before, "{name}");
        }
        assert!(import_from_file(&s, &d.join("missing"), true).is_err());
        assert_eq!(show(&s).unwrap().peer_id, before);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn other_key_types_and_passphrase_keys_are_refused_with_their_own_errors() {
        let d = tmp("refuse");
        let s = MemStore::new();
        let r = import_from_file(&s, &file(&d, "ec", ECDSA), false);
        assert!(
            matches!(&r, Err(KeyError::WrongKeyType(t)) if t == "ecdsa-sha2-nistp256"),
            "{r:?}"
        );
        let r = import_from_file(&s, &file(&d, "enc", ED_ENCRYPTED), false);
        assert!(matches!(r, Err(KeyError::Encrypted)));
        assert_eq!(stored(&s), None);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn malformed_files_are_refused() {
        let d = tmp("malformed");
        let s = MemStore::new();
        let lines: Vec<&str> = ED.lines().collect();
        let truncated = format!("{}\n{}\n{}\n{}\n", lines[0], lines[1], lines[2], END);
        let bad_base64 = format!("{BEGIN}\n!!!not base64!!!\n{END}\n");
        let with_trailing = format!("{ED}\nextra\n");
        let no_end = lines[..lines.len() - 1].join("\n");
        for (name, text) in [
            ("empty", String::new()),
            ("truncated", truncated),
            ("bad_base64", bad_base64),
            ("trailing", with_trailing),
            ("no_end", no_end),
            ("public_line", ED_PUBLIC.to_owned()),
        ] {
            let r = import_from_file(&s, &file(&d, name, &text), false);
            assert!(r.is_err(), "{name} was accepted");
        }
        assert_eq!(stored(&s), None);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_tampered_private_section_is_refused() {
        // Flip one byte inside the private section: the check numbers, the padding or the
        // public/private match must catch it.
        let d = tmp("tamper");
        let s = MemStore::new();
        let b64: String = ED.lines().filter(|l| !l.starts_with("-----")).collect();
        let mut bytes = BASE64.decode(b64.as_bytes()).unwrap();
        let n = bytes.len();
        bytes[n - 20] ^= 0x01;
        let text = format!("{BEGIN}\n{}\n{END}\n", BASE64.encode(&bytes));
        assert!(import_from_file(&s, &file(&d, "k", &text), false).is_err());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn crlf_line_endings_are_accepted() {
        let d = tmp("crlf");
        let s = MemStore::new();
        let crlf = ED.replace('\n', "\r\n");
        import_from_file(&s, &file(&d, "k", &crlf), false).unwrap();
        assert_eq!(show(&s).unwrap().openssh_public, ED_PUBLIC);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_huge_file_is_refused_unread() {
        let d = tmp("huge");
        let p = file(&d, "k", &"A".repeat(20_000));
        let r = import_from_file(&MemStore::new(), &p, false);
        assert!(matches!(r, Err(KeyError::NotOpenSsh)));
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn reset_changes_the_key_and_reports_the_old_one() {
        let s = MemStore::new();
        let r = reset(&s).unwrap();
        assert_eq!(r.old, OldKey::None);
        assert_eq!(show(&s).unwrap().peer_id, r.new);
        let r2 = reset(&s).unwrap();
        assert_eq!(r2.old, OldKey::Id(r.new));
        assert_ne!(r2.new, r.new);
        assert_eq!(show(&s).unwrap().peer_id, r2.new);
    }

    #[test]
    fn a_damaged_stored_key_blocks_show_but_reset_and_forced_import_repair_it() {
        let d = tmp("damaged");
        let s = MemStore::new();
        s.put(KEY_IDENTITY_SECRET, b"short").unwrap();
        assert!(matches!(show(&s), Err(KeyError::BadStoredKey)));
        assert!(matches!(export_text(&s), Err(KeyError::BadStoredKey)));
        let p = file(&d, "k", ED);
        match import_from_file(&s, &p, false) {
            Err(KeyError::WouldReplace(r)) => assert_eq!(r.old, OldKey::Unreadable),
            other => panic!("{other:?}"),
        }
        let r = import_from_file(&s, &p, true).unwrap();
        assert_eq!(r.old, OldKey::Unreadable);
        assert_eq!(show(&s).unwrap().peer_id, r.new);
        s.put(KEY_IDENTITY_SECRET, b"short").unwrap();
        let r = reset(&s).unwrap();
        assert_eq!(r.old, OldKey::Unreadable);
        assert_eq!(show(&s).unwrap().peer_id, r.new);
        let _ = std::fs::remove_dir_all(d);
    }
}
