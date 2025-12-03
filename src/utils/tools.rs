const ALPHABET: &[u8; 56] = b"23456789abcdefghijkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ";
const BASE: u64 = 56;

pub fn encode_base56(mut n: u64) -> String {
  if n == 0 {
    return (ALPHABET[0] as char).to_string();
  }
  let mut buf = Vec::new();
  while n > 0 {
    let i = (n% BASE) as usize;
    buf.push(ALPHABET[i]);
    n /= BASE;
  }
  buf.reverse();
  String::from_utf8(buf).unwrap()
}