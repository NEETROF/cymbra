//! Channels (design D4). A channel identifies a `(module, kind)` stream of jobs
//! and carries the ordering policy: ordered channels run strictly sequentially
//! (one at a time, in order); unordered channels run in parallel. The channel
//! *name* encodes `(module, kind)` so module/type separation is a column, not a
//! table (design D3).
//!
//! Pure, host-testable: no I/O. The sqlxmq glue reads [`Channel::name`] and
//! [`Ordering::is_ordered`] when building a job.

/// Whether jobs in a channel must run sequentially.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    /// Strictly sequential within the channel (sqlxmq `ordered = true`).
    Ordered,
    /// May run concurrently (sqlxmq `ordered = false`).
    Parallel,
}

impl Ordering {
    /// The sqlxmq `ordered` flag this policy maps to.
    pub fn is_ordered(self) -> bool {
        matches!(self, Ordering::Ordered)
    }
}

/// A `(module, kind)` job channel plus its ordering policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    module: String,
    kind: String,
    ordering: Ordering,
}

impl Channel {
    /// Build a channel for `(module, kind)` with an explicit ordering policy.
    pub fn new(module: impl Into<String>, kind: impl Into<String>, ordering: Ordering) -> Self {
        Self {
            module: module.into(),
            kind: kind.into(),
            ordering,
        }
    }

    /// Convenience: an ordered (sequential) channel.
    pub fn ordered(module: impl Into<String>, kind: impl Into<String>) -> Self {
        Self::new(module, kind, Ordering::Ordered)
    }

    /// Convenience: an unordered (parallel) channel.
    pub fn parallel(module: impl Into<String>, kind: impl Into<String>) -> Self {
        Self::new(module, kind, Ordering::Parallel)
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn ordering(&self) -> Ordering {
        self.ordering
    }

    pub fn is_ordered(&self) -> bool {
        self.ordering.is_ordered()
    }

    /// The sqlxmq `channel_name`: `"<module>.<kind>"`. Stable and human-readable
    /// so it can be filtered in the observability views and the worker's
    /// channel allow-list.
    pub fn name(&self) -> String {
        format!("{}.{}", self.module, self.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_encodes_module_and_kind() {
        assert_eq!(Channel::ordered("auth", "email").name(), "auth.email");
        assert_eq!(Channel::parallel("user", "reap").name(), "user.reap");
    }

    #[test]
    fn ordering_maps_to_sqlxmq_flag() {
        assert!(Channel::ordered("auth", "email").is_ordered());
        assert!(!Channel::parallel("auth", "webhook").is_ordered());
        assert!(Ordering::Ordered.is_ordered());
        assert!(!Ordering::Parallel.is_ordered());
    }

    #[test]
    fn accessors_return_parts() {
        let c = Channel::new("auth", "email", Ordering::Ordered);
        assert_eq!(c.module(), "auth");
        assert_eq!(c.kind(), "email");
        assert_eq!(c.ordering(), Ordering::Ordered);
    }
}
