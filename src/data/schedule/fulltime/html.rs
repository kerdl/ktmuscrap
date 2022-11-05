use derive_new::new;


#[derive(new, Debug, Clone)]
pub struct HeaderTable {
    pub header: String,
    pub table: Vec<Vec<String>>,
}
