pub fn bar(p: u32, q: u32) -> u32 {
    p + q
}

pub mod nested {
    pub fn bar(m: u32, n: u32) -> u32 {
        m + n
    }
}
