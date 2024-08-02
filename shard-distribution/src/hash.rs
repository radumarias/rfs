#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn hash(entity: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(entity.as_bytes());
    let result: [u8; 32] = hasher.finalize().into();
    u64::from_be_bytes(result[..8].try_into().unwrap())
}
