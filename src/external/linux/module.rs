pub struct PageProtection {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}
pub struct Page {
    pub base: usize,
    pub end: usize,
    pub protections: PageProtection,
}
pub struct ExModule<'a> {
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub process: &'a super::process::ExProcess,
    pub pages: Vec<Page>,
}
