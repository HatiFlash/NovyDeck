//! Protection des secrets stockés (mot de passe OBS) : DPAPI, portée utilisateur Windows.
//! Le texte chiffré n'est lisible que par le même compte Windows sur la même machine.
//! Hors Windows (compilation de développement uniquement), il n'existe aucun coffre : `protect` renvoie une erreur
//! et la sérialisation garde alors le texte clair, ce qui n'est jamais le cas dans l'application Windows livrée.
use base64::{engine::general_purpose::STANDARD, Engine};

#[cfg(windows)]
pub fn protect(plain: &str) -> Result<String, String> {
    use windows_sys::Win32::{Foundation::LocalFree, Security::Cryptography::{CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB}};
    let input = CRYPT_INTEGER_BLOB { cbData: plain.len() as u32, pbData: plain.as_ptr() as *mut u8 };
    let mut output = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
    // SAFETY: `input` référence `plain` pendant l'appel ; `output` est alloué par DPAPI et libéré par LocalFree.
    unsafe {
        if CryptProtectData(&input, std::ptr::null(), std::ptr::null(), std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut output) == 0 {
            return Err("Chiffrement DPAPI impossible".into());
        }
        let bytes = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData as _);
        Ok(STANDARD.encode(bytes))
    }
}
#[cfg(windows)]
pub fn unprotect(encoded: &str) -> Result<String, String> {
    use windows_sys::Win32::{Foundation::LocalFree, Security::Cryptography::{CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB}};
    let bytes = STANDARD.decode(encoded.trim()).map_err(|_| "Secret illisible")?;
    let input = CRYPT_INTEGER_BLOB { cbData: bytes.len() as u32, pbData: bytes.as_ptr() as *mut u8 };
    let mut output = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
    // SAFETY: voir `protect`.
    unsafe {
        if CryptUnprotectData(&input, std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut output) == 0 {
            return Err("Secret non déchiffrable avec ce compte Windows".into());
        }
        let plain = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData as _);
        String::from_utf8(plain).map_err(|_| "Secret illisible".into())
    }
}
#[cfg(not(windows))]
pub fn protect(_plain: &str) -> Result<String, String> { let _ = STANDARD; Err("DPAPI indisponible hors Windows".into()) }
#[cfg(not(windows))]
pub fn unprotect(_encoded: &str) -> Result<String, String> { Err("DPAPI indisponible hors Windows".into()) }

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test] fn dpapi_round_trip_hides_the_secret() {
        for secret in ["motdepasse-OBS", "é ü 日本語 !\"#", ""] {
            let sealed = protect(secret).unwrap();
            assert!(secret.is_empty() || !sealed.contains(secret));
            assert_eq!(unprotect(&sealed).unwrap(), secret);
        }
        assert!(unprotect("pas du base64 !").is_err());
        assert!(unprotect(&STANDARD.encode(b"donnees quelconques")).is_err());
    }
}
