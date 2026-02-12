use crate::*;
use pyo3::prelude::*;

#[pyclass]
pub struct Signer {
    inner: key::Signer,
}

#[pymethods]
impl Signer {
    #[new]
    pub fn new(private_key_hex: &str, public_key_hex: &str) -> PyResult<Self> {
        Ok(Self {
            inner: key::Signer::new_with_hexes(private_key_hex, public_key_hex)?,
        })
    }

    pub fn sign(&self, message: &[u8]) -> PyResult<String> {
        Ok(self.inner.sign(message)?)
    }
}

#[pyclass]
pub struct Verifier {
    inner: key::Verifier,
}

#[pymethods]
impl Verifier {
    #[new]
    pub fn new(public_key_hexes: Vec<String>) -> PyResult<Self> {
        Ok(Self {
            inner: key::Verifier::new(&public_key_hexes)?,
        })
    }

    pub fn verify(&self, message: &[u8], signature_base64: &str) -> PyResult<()> {
        Ok(self.inner.verify(message, signature_base64)?)
    }
}
