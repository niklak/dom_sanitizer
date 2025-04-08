use crate::policy::{Policy, SanitizeDirective};
use dom_query::{Document, NodeRef};

pub trait SanitizeExt {
    fn sanitize<T: SanitizeDirective>(&self, policy: &Policy<T>);
}

impl SanitizeExt for NodeRef<'_> {
    fn sanitize<T: SanitizeDirective>(&self, policy: &Policy<T>) {
        policy.sanitize_node(self);
    }
}

impl SanitizeExt for Document {
    fn sanitize<T: SanitizeDirective>(&self, policy: &Policy<T>) {
        policy.sanitize_document(self);
    }
}
