#[derive(Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub title: String,
    pub version: String,
    pub linux: String,
    pub initrd: String,
    pub options: String,
}
