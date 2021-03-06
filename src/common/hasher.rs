use base16ct;
use sha3::{Digest, Keccak256};

pub fn hash(data: &str) -> String {
  let mut hasher = Keccak256::new();
  hasher.update(data);

  let result = hasher.finalize_reset();
  let mut buf = [0u8; 64];
  let res: &str = base16ct::lower::encode_str(&result, &mut buf).unwrap();
  return res.to_string();
}
