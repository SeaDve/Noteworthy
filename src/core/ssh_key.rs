#[derive(Debug)]
pub struct SshKey {
    pub public_key: String,
    pub private_key: String,
    pub password: String,
}

impl SshKey {
    pub fn generate(password: &str) -> anyhow::Result<Self> {
        let rsa_key = openssl::rsa::Rsa::generate(4096)?;

        let public_key = rsa_key.public_key_to_pem()?;
        let private_key = rsa_key.private_key_to_pem()?;

        Ok(SshKey {
            public_key: String::from_utf8(public_key)?,
            private_key: String::from_utf8(private_key)?,
            password: password.to_string(),
        })
    }

    pub fn generate_default() -> anyhow::Result<Self> {
        Self::generate("")
    }
}
