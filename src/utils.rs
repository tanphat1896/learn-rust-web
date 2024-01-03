use argon2::Config;
use rand::{thread_rng, Rng};

pub(crate) fn hash_password(raw: String) -> String {
  let bytes = raw.as_bytes();
  let mut rng = thread_rng();
  let salt: [u8; 20] = rng.gen();
  argon2::hash_encoded(bytes, &salt, &Config::default()).expect("Failed to hash password")
}

pub(crate) fn check_password(raw: String, hashed: String) -> bool {
  argon2::verify_encoded(hashed.as_str(), raw.as_bytes()).unwrap_or(false)
}
