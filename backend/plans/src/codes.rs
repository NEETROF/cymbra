//! Access-code generation and hashing (design D4): ≥128 bits of CSPRNG, a
//! human-typeable clear text shown once, and a SHA-256 hash at rest.

use sha2::{Digest, Sha256};

/// Crockford-style base32 alphabet without the ambiguous I/L/O/U.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// 26 symbols × 5 bits = 130 bits of entropy.
const CODE_SYMBOLS: usize = 26;

/// A fresh clear-text code, grouped as `XXXXX-XXXXX-XXXXX-XXXXX-XXXXX-X`
/// (26 symbols → 130 bits).
pub fn generate_code() -> String {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("OS CSPRNG unavailable for access code");
    let symbols: String = bytes
        .iter()
        .take(CODE_SYMBOLS)
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect();
    group(&symbols)
}

fn group(symbols: &str) -> String {
    symbols
        .as_bytes()
        .chunks(5)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("-")
}

/// Canonical form: uppercase, dashes/spaces removed, ambiguous glyphs mapped
/// (`O→0`, `I/L→1`) so a code read aloud still matches.
pub fn normalize(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| match c.to_ascii_uppercase() {
            'O' => '0',
            'I' | 'L' => '1',
            other => other,
        })
        .collect()
}

/// SHA-256 hex of the normalized code — the only thing stored.
pub fn hash_code(input: &str) -> String {
    let normalized = normalize(input);
    let digest = Sha256::digest(normalized.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_codes_are_long_unique_and_typeable() {
        let a = generate_code();
        let b = generate_code();
        assert_ne!(a, b);
        assert_eq!(normalize(&a).len(), CODE_SYMBOLS);
        assert!(normalize(&a).chars().all(|c| ALPHABET.contains(&(c as u8))));
        assert_eq!(a.matches('-').count(), 5);
    }

    #[test]
    fn hash_is_stable_across_formatting_and_ambiguous_glyphs() {
        let h1 = hash_code("ABCDE-FGH1J-KMNPQ-RSTVW-XYZ01-2");
        let h2 = hash_code(" abcde fgh1j kmnpq rstvw xyzo1 2 ");
        let h3 = hash_code("ABCDEFGHIJKMNPQRSTVWXYZOl2");
        assert_eq!(h1, h2);
        assert_eq!(h1, h3);
        assert_eq!(h1.len(), 64);
        assert_ne!(h1, hash_code("ABCDE-FGH1J-KMNPQ-RSTVW-XYZ01-3"));
    }
}
