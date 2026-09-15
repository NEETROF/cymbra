//! The client address a request is attributed to (change: fix-auth-lockout-dos), resolved
//! the way the deployment presents it: Caddy is the only public entry and rewrites the
//! forwarded headers, so its first `X-Forwarded-For` hop is the real client. Shared by the
//! gRPC and HTTP adapters so every rate limit keys the same address.

use std::net::IpAddr;

/// The address used when none could be read (every such request shares one budget).
pub const UNKNOWN: &str = "unknown";

/// Resolve the client address from a header getter and the transport's peer address: the
/// first non-empty `X-Forwarded-For` hop, else `X-Real-IP`, else `peer`, else [`UNKNOWN`].
pub fn resolve(header: impl Fn(&str) -> Option<String>, peer: Option<IpAddr>) -> String {
    let non_empty = |v: String| {
        let v = v.trim().to_string();
        (!v.is_empty()).then_some(v)
    };
    header("x-forwarded-for")
        .and_then(|v| v.split(',').next().map(str::to_string))
        .and_then(non_empty)
        .or_else(|| header("x-real-ip").and_then(non_empty))
        .or_else(|| peer.map(|ip| ip.to_string()))
        .unwrap_or_else(|| UNKNOWN.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn resolve_with(h: &HashMap<String, String>, peer: Option<IpAddr>) -> String {
        resolve(|name| h.get(name).cloned(), peer)
    }

    #[test]
    fn prefers_the_first_forwarded_hop() {
        let h = headers(&[
            ("x-forwarded-for", " 203.0.113.5 , 10.0.0.1"),
            ("x-real-ip", "10.0.0.9"),
        ]);
        assert_eq!(
            resolve_with(&h, Some("10.0.0.2".parse().unwrap())),
            "203.0.113.5"
        );
    }

    #[test]
    fn falls_back_to_real_ip_then_peer_then_unknown() {
        let peer: IpAddr = "10.0.0.2".parse().unwrap();
        assert_eq!(
            resolve_with(&headers(&[("x-real-ip", "10.0.0.9")]), Some(peer)),
            "10.0.0.9"
        );
        assert_eq!(resolve_with(&headers(&[]), Some(peer)), "10.0.0.2");
        assert_eq!(resolve_with(&headers(&[]), None), UNKNOWN);
    }

    #[test]
    fn skips_empty_header_values() {
        let h = headers(&[("x-forwarded-for", " , 10.0.0.1"), ("x-real-ip", "  ")]);
        assert_eq!(resolve_with(&h, None), UNKNOWN);
    }
}
