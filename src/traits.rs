use dom_query::NodeRef;

/// A trait for sanitization directives, defines methods for node and attribute sanitization.
pub trait SanitizeDirective {
    /// Sanitizes a node by removing elements and attributes based on the policy.
    fn sanitize_node(policy: &impl SanitizePolicy, node: &NodeRef)
    where
        Self: Sized;
    /// Sanitizes the attributes of a node by removing or retaining them based on the policy.
    fn sanitize_node_attrs(policy: &impl SanitizePolicy, node: &dom_query::NodeRef)
    where
        Self: Sized;
}

/// A trait that defines a sanitization policy.
pub trait SanitizePolicy {
    /// Whether node should be excluded from the sanitization process.
    fn should_exclude(&self, node: &NodeRef) -> bool;
    /// Whether node should be removed from the DOM.
    fn should_remove(&self, node: &NodeRef) -> bool;
    /// Whether the policy has attributes to be excluded.
    fn has_attrs_to_exclude(&self) -> bool;
    /// Excludes the attributes of a node based on the policy.
    fn exclude_attrs<F>(&self, node: &NodeRef, exclude_fn: F)
    where
        F: FnOnce(&NodeRef, &[&str]);
    /// A policy instance doesn't have any special exclusions.
    fn is_empty(&self) -> bool;
}
