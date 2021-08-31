#[derive(Clone)]
pub struct Iv(pub String);
impl Iv {
    pub fn new(new: String) -> Self {
        Self(new)
    }
}

#[derive(Clone)]
pub struct AesKeyEncrypt(pub Vec<u8>);
impl AesKeyEncrypt {
    pub fn new(new: Vec<u8>) -> Self {
        Self(new)
    }
}

pub struct AesKey(pub String);
impl AesKey {
    pub fn new(new: String) -> Self {
        Self(new)
    }
}

#[derive(Clone)]
pub struct EncryptedBody(pub Vec<u8>);

impl EncryptedBody {
    pub fn new(new: Vec<u8>) -> Self {
        Self(new)
    }
}

pub struct Body(pub String);

impl Body {
    pub fn new(new: String) -> Self {
        Self(new)
    }
}