use super::core::Policy;
use crate::traits::SanitizeDirective;

use dom_query::{Document, NodeRef};

/// A trait that provides a method to sanitize a DOM node or document
pub trait SanitizeExt {
    fn sanitize<T: SanitizeDirective>(&self, policy: &Policy<T>);
}

impl SanitizeExt for NodeRef<'_> {
    /// Sanitizes the node using the provided policy.
    fn sanitize<T: SanitizeDirective>(&self, policy: &Policy<T>) {
        policy.sanitize_node(self);
    }
}

impl SanitizeExt for Document {
    /// Sanitizes the document using the provided policy.
    fn sanitize<T: SanitizeDirective>(&self, policy: &Policy<T>) {
        policy.sanitize_document(self);
    }
}
