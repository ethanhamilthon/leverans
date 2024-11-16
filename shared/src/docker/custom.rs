#[derive(Debug, Clone)]
pub struct CustomApi {
    pub url: String,
}

impl CustomApi {
    pub fn new(url: String) -> Self {
        CustomApi { url }
    }
}
