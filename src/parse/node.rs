pub mod text {
    use html_parser::Node;
    use htmlescape::decode_html;

    pub fn nested_as_vec(node: &Node) -> Vec<String> {
        let mut texts = vec![];

        if node.text().is_some() {
            if let Ok(decoded) = decode_html(node.text().unwrap()) {
                texts.push(decoded)
            } else {
                texts.push(node.text().unwrap().to_owned())
            }
        }

        if node.element().is_none() {
            return texts
        }

        for child in node.element().unwrap().children.iter() {
            let child_text = nested_as_vec(child);
            texts.extend_from_slice(&child_text)
        }

        texts
    }

    /// # Get text from all child nodes
    /// 
    /// ## Example input
    /// 
    /// ```notrust
    /// node = {
    ///     < td >
    ///         text1 
    ///         < br >
    ///         text2
    ///         < span >
    ///             text3
    ///         < /span >
    ///     < /td >
    /// }
    /// sep = ", "
    /// ```
    /// ## Example output
    /// - `"text1, text2, text3"`
    pub fn nested_as_string(node: &Node, sep: &str) -> String {
        let texts = nested_as_vec(node);
        texts.join(sep)
    }
}
