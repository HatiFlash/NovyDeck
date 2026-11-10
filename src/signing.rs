//! Signatures Ed25519 des mises à jour et du catalogue (S-01, S-02) : deux clés publiques épinglées, l'une ou l'autre est acceptée.
#![cfg(windows)]
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, VerifyingKey};

const PRIMARY: &str = "wDsBxy02b5CcI0CbNCQe0gHooUDH/j+K+TUIDqXSo4Q=";
const BACKUP: &str = "VqvMDqAPWMpjaD0GeHSowJ7A6snekH4NQDvNwAFRjeg=";
pub const SIG_LIMIT: usize = 512;

fn verify_with(keys: &[&str], message: &[u8], sig_text: &str) -> Result<()> {
    let compact: String = sig_text.split_whitespace().collect();
    let raw = STANDARD
        .decode(compact.as_bytes())
        .context("Signature illisible (base64 invalide)")?;
    let sig = Signature::from_slice(&raw)
        .map_err(|_| anyhow::anyhow!("Signature illisible (longueur incorrecte)"))?;
    for key in keys {
        let Ok(bytes) = STANDARD.decode(key) else {
            continue;
        };
        let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice()) else {
            continue;
        };
        let Ok(key) = VerifyingKey::from_bytes(&bytes) else {
            continue;
        };
        if key.verify_strict(message, &sig).is_ok() {
            return Ok(());
        }
    }
    anyhow::bail!("Signature invalide")
}
fn update_message(version: &str, sha256: &str) -> Vec<u8> {
    format!(
        "NovyDeck update v1\n{}\n{}\n",
        version.trim_start_matches('v'),
        sha256.trim().to_ascii_lowercase()
    )
    .into_bytes()
}
fn registry_message(bytes: &[u8]) -> Vec<u8> {
    [b"NovyDeck registry v1\n".as_slice(), bytes].concat()
}

/// `version` avec ou sans « v » ; `sha256` en hexadécimal, majuscules ou minuscules.
pub fn verify_update(version: &str, sha256: &str, sig_text: &str) -> Result<()> {
    verify_with(
        &[PRIMARY, BACKUP],
        &update_message(version, sha256),
        sig_text,
    )
}
pub fn verify_registry(bytes: &[u8], sig_text: &str) -> Result<()> {
    verify_with(&[PRIMARY, BACKUP], &registry_message(bytes), sig_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }
    fn public(k: &SigningKey) -> String {
        STANDARD.encode(k.verifying_key().to_bytes())
    }
    fn sign(k: &SigningKey, message: &[u8]) -> String {
        STANDARD.encode(k.sign(message).to_bytes())
    }
    const SHA: &str = "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF";

    #[test]
    fn valid_with_either_key_and_unknown_key_refused() {
        let (a, b, c) = (key(1), key(2), key(3));
        let keys = [public(&a), public(&b)];
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        let msg = update_message("0.19.3", SHA);
        verify_with(&keys, &msg, &sign(&a, &msg)).unwrap();
        verify_with(&keys, &msg, &sign(&b, &msg)).unwrap();
        assert!(verify_with(&keys, &msg, &sign(&c, &msg)).is_err());
        assert!(verify_with(&[], &msg, &sign(&a, &msg)).is_err());
    }
    #[test]
    fn modified_message_refused() {
        let a = key(1);
        let pk = public(&a);
        let keys = [pk.as_str()];
        let sig = sign(&a, &update_message("0.19.3", SHA));
        verify_with(&keys, &update_message("0.19.3", SHA), &sig).unwrap();
        assert!(verify_with(&keys, &update_message("0.19.4", SHA), &sig).is_err());
        assert!(verify_with(
            &keys,
            &update_message("0.19.3", &SHA.replace('0', "1")),
            &sig
        )
        .is_err());
        let reg = b"{\"schema\":1,\"plugins\":[]}".to_vec();
        let sig = sign(&a, &registry_message(&reg));
        verify_with(&keys, &registry_message(&reg), &sig).unwrap();
        let mut altered = reg.clone();
        altered[3] ^= 1;
        assert!(verify_with(&keys, &registry_message(&altered), &sig).is_err());
        assert!(verify_with(
            &keys,
            &registry_message(&reg),
            &sign(&a, &update_message("0.19.3", SHA))
        )
        .is_err());
    }
    #[test]
    fn malformed_signature_refused() {
        let a = key(1);
        let pk = public(&a);
        let keys = [pk.as_str()];
        let msg = update_message("0.19.3", SHA);
        let sig = sign(&a, &msg);
        for bad in [
            "",
            "!!!not base64!!!",
            "AAAA",
            &sig[..sig.len() - 4],
            &format!("{sig}AAAA"),
        ] {
            assert!(verify_with(&keys, &msg, bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn whitespace_tolerated_and_hash_case_normalised() {
        let a = key(1);
        let pk = public(&a);
        let keys = [pk.as_str()];
        let msg = update_message("0.19.3", &SHA.to_ascii_lowercase());
        let sig = sign(&a, &msg);
        let (head, tail) = sig.split_at(20);
        verify_with(&keys, &msg, &format!("  {head}\r\n{tail}\n\n")).unwrap();
        assert_eq!(update_message("v0.19.3", SHA), msg);
        assert_eq!(
            msg,
            format!("NovyDeck update v1\n0.19.3\n{}\n", SHA.to_ascii_lowercase()).into_bytes()
        );
    }
    #[test]
    fn pinned_keys_are_valid_and_fixed_vector_verifies() {
        for k in [PRIMARY, BACKUP] {
            let bytes: [u8; 32] = STANDARD.decode(k).unwrap().try_into().unwrap();
            VerifyingKey::from_bytes(&bytes).unwrap();
        }
        let sig = "7a2ouLXVU9rMepZFt15oZmi5Se8GKndzkVme0FyNCFwEoT9qqbR6AH1J+QOnqX8fepm0LIQPlXDla2NM9palDg==";
        verify_with(&[PRIMARY, BACKUP], b"NovyDeck test v1\n", sig).unwrap();
        assert!(verify_with(&[PRIMARY], b"NovyDeck test v2\n", sig).is_err());
        assert!(verify_update("0.19.3", SHA, sig).is_err());
        assert!(verify_registry(b"{}", sig).is_err());
    }
}
