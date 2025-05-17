macro_rules! sanitize_methods {
    () => {
        /// Sanitizes a node by applying the policy rules according to the directive type.
        ///
        /// For [Permissive] directive: Removes elements and attributes specified in the policy.
        /// For [Restrictive] directive: Keeps only elements and attributes specified in the policy.
        pub fn sanitize_node(&self, node: &dom_query::NodeRef) {
            T::sanitize_node(self, node);
            node.normalize();
        }

        /// Sanitizes the [dom_query::Document].
        pub fn sanitize_document(&self, document: &dom_query::Document) {
            self.sanitize_node(&document.root());
        }

        /// Sanitizes the [dom_query::Selection].
        pub fn sanitize_selection(&self, sel: &dom_query::Selection) {
            for node in sel.nodes() {
                self.sanitize_node(node);
            }
        }

        /// Sanitizes the HTML content by applying the policy rules according to the directive type.
        pub fn sanitize_html<S: Into<StrTendril>>(&self, html: S) -> StrTendril {
            let doc = Document::from(html);
            self.sanitize_document(&doc);
            doc.html()
        }
    };
}

pub(crate) use sanitize_methods;
