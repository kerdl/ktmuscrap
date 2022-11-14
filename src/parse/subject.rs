pub fn is_split(string: &str) -> bool {
    string.contains("/")
}

pub fn split(string: &str) -> Vec<String> {
    string.split("/").map(|string| string.trim().to_owned()).collect()
}