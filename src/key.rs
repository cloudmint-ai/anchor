use crate::*;
use hex;
use num_bigint::BigUint;
use rand_core::OsRng;
use sm2::{
    SecretKey,
    dsa::{Signature, SigningKey, VerifyingKey, signature::Signer as _, signature::Verifier as _},
};

// TODO change
const DISTID: &str = "1234567812345678";

#[derive(Clone)]
pub struct Signer {
    private_key: PrivateKey,
    public_key: PublicKey,
}

impl Signer {
    pub fn new() -> Result<Self> {
        Self::new_with_hexes(&config!().private_key_hex, &config!().public_key_hex)
    }

    // 尽量避免私钥 hex 的clone
    pub(crate) fn new_with_hexes(private_key_hex: &str, public_key_hex: &str) -> Result<Self> {
        let private_key = PrivateKey::new(private_key_hex)?;
        let public_key = PublicKey::new(public_key_hex)?;
        verify_key_pair_matching(&private_key, &public_key)?;
        Ok(Self {
            private_key,
            public_key,
        })
    }

    pub fn sign(&self, message: &[u8]) -> Result<String> {
        Ok(format!(
            "{}.{}",
            self.public_key.brief(),
            self.private_key.sign(message)?
        ))
    }
}

// TODO , use Arc<dyn get_public_key with public_key_biref>
#[derive(Clone)]
pub struct Verifier {
    public_keys: HashMap<String, Vec<PublicKey>>,
}

impl Verifier {
    pub fn new(public_key_hexes: &Vec<String>) -> Result<Self> {
        let mut public_keys = HashMap::new();

        for public_key_hex in public_key_hexes.into_iter() {
            let public_key = PublicKey::new(public_key_hex)?;

            public_keys
                .entry(public_key.brief())
                .or_insert_with(Vec::new)
                .push(public_key);
        }
        Ok(Self { public_keys })
    }

    pub fn verify(&self, message: &[u8], signature_base64: &str) -> Result<()> {
        let signature_parts: Vec<&str> = signature_base64.split(".").collect();
        if signature_parts.len() != 2 {
            return Unexpected!("signature without . not supported");
        }
        let public_key_brief = signature_parts[0];
        let signature_base64 = signature_parts[1];

        for public_key in self.public_keys.get(public_key_brief).ok_or_else(none!())? {
            if let Ok(()) = public_key.verify(message, signature_base64) {
                return Ok(());
            }
        }

        Unexpected!("message verify fail")
    }
}

#[derive(Clone)]
pub struct PublicKey(VerifyingKey);

impl PublicKey {
    pub fn new(hex: &str) -> Result<PublicKey> {
        let bytes = hex::decode(hex)?;
        let verifying_key = VerifyingKey::from_sec1_bytes(DISTID, &bytes)?;
        Ok(PublicKey(verifying_key))
    }

    pub fn verify(&self, message: &[u8], signature_base64: &str) -> Result<()> {
        let signature: Vec<u8> = base64::decode(signature_base64)?;
        let signature = asn1_decode(signature)?;
        let signature = Signature::from_bytes(&signature)?;
        Ok(self.0.verify(message, &signature)?)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0.to_sec1_bytes())
    }

    pub fn brief(&self) -> String {
        base64::encode(self.0.to_sec1_bytes())[..11].to_string()
    }
}

#[derive(Clone)]
pub struct PrivateKey(SigningKey);

impl PrivateKey {
    // 尽量避免私钥 hex 的clone
    pub fn new(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex)?;
        let secret_key = SecretKey::from_slice(&bytes)?;
        let signing_key = SigningKey::new(DISTID, &secret_key)?;
        Ok(Self(signing_key))
    }

    pub fn sign(&self, message: &[u8]) -> Result<String> {
        let signature: Signature = self.0.sign(message);
        let signature = asn1_encode(signature.to_bytes())?;
        Ok(base64::encode(signature))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0.to_bytes())
    }
}

pub fn generate_key() -> Result<(PublicKey, PrivateKey)> {
    let secret_key = SecretKey::random(&mut OsRng);
    let distid = DISTID;
    let signing_key = SigningKey::new(distid, &secret_key)?;
    let verifying_key = signing_key.verifying_key().clone();
    Ok((PublicKey(verifying_key), PrivateKey(signing_key)))
}

pub fn verify_key_pair_matching(private_key: &PrivateKey, public_key: &PublicKey) -> Result<()> {
    let test_message = b"test message";
    let signature = private_key.sign(test_message)?;
    public_key.verify(test_message, &signature)?;
    Ok(())
}

fn pad_bytes_with_0_prefix(bytes: &[u8], target_length: usize) -> Vec<u8> {
    let mut padded = vec![0u8; target_length];
    let len = bytes.len();

    if len < target_length {
        padded[target_length - len..target_length].copy_from_slice(bytes);
    } else {
        padded.copy_from_slice(&bytes[..target_length]);
    }

    padded
}

fn asn1_decode(bytes: Vec<u8>) -> Result<[u8; 64]> {
    let (r, s) = yasna::parse_der(&bytes, |reader| {
        reader.read_sequence(|reader| {
            let r = reader.next().read_biguint()?;
            let s = reader.next().read_biguint()?;
            Ok((r, s))
        })
    })
    .map_err(|e| unexpected!("asn1 encode fail {:?}", e))?;

    let mut bytes = [0u8; 64];
    let mut r_bytes = r.to_bytes_be();
    let mut s_bytes = s.to_bytes_be();
    if r_bytes.len() != 32 {
        r_bytes = pad_bytes_with_0_prefix(&r_bytes, 32)
    }
    if s_bytes.len() != 32 {
        s_bytes = pad_bytes_with_0_prefix(&s_bytes, 32)
    }
    bytes[0..32].copy_from_slice(&r_bytes);
    bytes[32..64].copy_from_slice(&s_bytes);
    Ok(bytes)
}

fn asn1_encode(bytes: [u8; 64]) -> Result<Vec<u8>> {
    let r_bytes = &bytes[0..32];
    let s_bytes = &bytes[32..64];

    let bytes = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer
                .next()
                .write_biguint(&BigUint::from_bytes_be(r_bytes));
            writer
                .next()
                .write_biguint(&BigUint::from_bytes_be(s_bytes));
        });
    });
    Ok(bytes)
}

#[cfg(feature = "runtime")]
tests! {
    fn test_key_pair_match() {
        let private_key =
            PrivateKey::new("b9ab0b828ff68872f21a837fc303668428dea11dcd1b24429d0c99e24eed83d5")?;
        let public_key = PublicKey::new(
            "04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207",
        )?;
        verify_key_pair_matching(&private_key, &public_key)?;
    }

    async fn test_message_verify() {
        let public_key = PublicKey::new(
            "04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207",
        )?;

        let message = br#"{"ProjectID":"123","Timestamp":1709955225448,"ID":"request_id_test","DataURL":"what"}"#;
        let signature_base64 = "MEQCIBNL6Y/E9QJEIiwU51AxWS9nz/KAyQrme5IYGo8Scl0lAiB8VorpOvuyybPNILHVwPi6a//NW+x92xxoQHohwz6l0Q==";

        public_key.verify(message, signature_base64)?;

        let signature_base64 = "MEQCIDvgDnkbzUVSM2dGOLODvKV5K/+976lVPajN537npbdaAiA5RmIw6O5QGIuajyUA2yQJbEL1aIarEbpJpmHFmS8oMA==";
        public_key.verify(message, signature_base64)?;

        assert_eq!(public_key.brief(), "BLnJpuBOnJH");
        let signature_base64 = &format!("{}.{}", public_key.brief(), signature_base64);

        let verifier = Verifier::new(&vec!["04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207".to_string()])?;
        verifier.verify(message, signature_base64)?;
    }

    #[cfg(feature = "api")]
    async fn test_decode_signature_len_not_32() {
        let data_bytes =
            br#"{"ProjectID":"123","Timestamp":1727115512682,"RequestID":"493486342907363328"}"#;
        let public_key = PublicKey::new(
            "04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207",
        )?;

        let signature_base64 = "MEQCIQDAttcj352DJRfZcZYMpFAXE5y9wK46O9VwbIWFLTW34QIfYhYObYP//4qRCiGz+7C5be9lq4g9mNQW451PbMNedQ==";
        public_key.verify(data_bytes, &signature_base64)?;
    }

    #[ignore]
    fn test_generate_key() {
        let (public_key, private_key) = generate_key()?;
        println!("publicKey:{}", public_key.to_hex());
        println!("privateKey:{}", private_key.to_hex());
    }
}
