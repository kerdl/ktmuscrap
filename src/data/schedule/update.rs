#[derive(Debug)]
pub enum Invoker {
    Auto,
    Manually(String)
}

#[derive(Debug)]
pub struct Params {
    pub invoker: Invoker,
}