use derive_new::new;


#[derive(new)]
pub struct HeaderTable {
    pub header: String,
    pub table: Vec<Vec<String>>,
}
