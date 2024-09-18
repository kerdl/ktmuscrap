pub mod path {
    use std::path::PathBuf;
    
    /// Returns a relative path without reserved names, redundant separators, ".", or "..".
    pub fn sanitize(path: &str) -> PathBuf {
        // Replaces backwards slashes
        path.replace('\\', "/")
            // Sanitizes each component
            .split('/')
            .map(sanitize_filename::sanitize)
            .collect()
}
}