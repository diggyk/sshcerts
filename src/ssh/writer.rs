use std::collections::HashMap;

use super::pubkey::{PublicKey, PublicKeyKind};

use byteorder::{BigEndian, ByteOrder};

/// A `Writer` is used for encoding a key in OpenSSH compatible format.
#[derive(Debug)]
pub struct Writer {
    inner: Vec<u8>,
}

impl Writer {
    /// Creates a new `Writer` instance.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// let writer = Writer::new();
    /// ```
    pub fn new() -> Writer {
        Writer { inner: Vec::new() }
    }

    /// Writes a byte sequence to the underlying vector.
    /// The value is represented as a the byte sequence length,
    /// followed by the actual byte sequence.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// let mut writer = Writer::new();
    /// writer.write_bytes(&[0, 0, 0, 42]);
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, vec![0, 0, 0, 4, 0, 0, 0, 42]);
    /// ```
    pub fn write_bytes(&mut self, val: &[u8]) {
        let size = val.len() as u32;
        let mut buf = vec![0; 4];
        BigEndian::write_u32(&mut buf, size);
        self.inner.append(&mut buf);
        self.inner.extend_from_slice(&val);
    }

    /// Writes a `string` value to the underlying byte sequence.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// let mut writer = Writer::new();
    /// writer.write_string("a test string");
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, [0, 0, 0, 13, 97, 32, 116, 101, 115, 116, 32, 115, 116, 114, 105, 110, 103]);
    /// ```
    pub fn write_string(&mut self, val: &str) {
        self.write_bytes(val.as_bytes());
    }

    /// Writes a `u64` value to the underlying byte sequence.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// let mut writer = Writer::new();
    /// writer.write_u64(0xFFFFFFFFFFFFFFFF);
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, [255, 255, 255, 255, 255, 255, 255, 255]);
    /// ```
    pub fn write_u64(&mut self, val: u64) {
        let bytes = val.to_be_bytes();
        self.inner.extend_from_slice(&bytes);
    }

    /// Writes a `u32` value to the underlying byte sequence.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// let mut writer = Writer::new();
    /// writer.write_u32(0xFFFFFFFF);
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, [255, 255, 255, 255]);
    /// ```
    pub fn write_u32(&mut self, val: u32) {
        let bytes = val.to_be_bytes();
        self.inner.extend_from_slice(&bytes);
    }

    /// Writes an `mpint` value to the underlying byte sequence.
    /// If the MSB bit of the first byte is set then the number is
    /// negative, otherwise it is positive.
    /// Positive numbers must be preceeded by a leading zero byte according to RFC 4251, section 5.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// let mut writer = Writer::new();
    /// writer.write_mpint(&[1, 0, 1]);
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, [0, 0, 0, 3, 1, 0, 1]);
    /// ```
    pub fn write_mpint(&mut self, val: &[u8]) {
        let mut bytes = val.to_vec();

        // If most significant bit is set then prepend a zero byte to
        // avoid interpretation as a negative number.
        if val.get(0).unwrap_or(&0) & 0x80 != 0 {
            bytes.insert(0, 0);
        }

        self.write_bytes(&bytes);
    }

    /// Writes a `Vec<String>` to the underlying byte sequence.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// let mut writer = Writer::new();
    /// 
    /// writer.write_string_vec(&vec![String::from("Test"), String::from("Test")]);
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, [0, 0, 0, 16, 0, 0, 0, 4, 84, 101, 115, 116, 0, 0, 0, 4, 84, 101, 115, 116]);
    /// ```
    pub fn write_string_vec(&mut self, vec: &[String]) {
        let total_length = vec.iter().map(|x| x.len()).fold(vec.len()*4, |x, y| x + y) as u32;
        self.write_u32(total_length);

        for item in vec {
            self.write_string(item);
        }
    }

    /// Writes a `HashMap<String, String>` to the underlying byte sequence.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// # use std::collections::HashMap;
    /// 
    /// let mut writer = Writer::new();
    /// let mut example_map = HashMap::new();
    /// example_map.insert(String::from("Test"), String::from(""));
    /// writer.write_string_map(&example_map);
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, [0, 0, 0, 12, 0, 0, 0, 4, 84, 101, 115, 116, 0, 0, 0, 0]);
    /// ```
    pub fn write_string_map(&mut self, map: &HashMap<String, String>) {
        let total_length = map.iter()
            .map(|x| x.0.len() + x.1.len() + if !x.1.is_empty() { 4 } else { 0 })
            .fold(map.len() * 8, |x, y| x + y) as u32;

        self.write_u32(total_length);

        for (k,v) in map {
            self.write_string(k);
            if v.is_empty() {
                self.write_u32(0x0);
            } else {
                self.write_u32(v.len() as u32 + 4);
                self.write_string(v);
            }
        }
    }

    /// Writes a `PublicKey` to the underlying byte sequence.
    ///
    /// # Example
    /// ```
    /// ```
    pub fn write_pub_key(&mut self, key: &PublicKey) {
         // Write the public key
         match &key.kind {
            PublicKeyKind::Ecdsa(key) => {
                self.write_string(key.curve.identifier);
                self.write_bytes(&key.key);
            },
            PublicKeyKind::Rsa(key) => {
                self.write_bytes(&key.n);
                self.write_bytes(&key.e);
            },
            PublicKeyKind::Ed25519(key) => {
                self.write_bytes(&key.key);
            }
        };
    }

    /// Converts the `Writer` into a byte sequence.
    /// This consumes the underlying byte sequence used by the `Writer`.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// 
    /// let mut writer = Writer::new();
    /// writer.write_string("some data");
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, [0, 0, 0, 9, 115, 111, 109, 101, 32, 100, 97, 116, 97]);
    /// ```
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner
    }

    /// Converts the `Writer` into a byte sequence.
    /// This consumes the underlying byte sequence used by the `Writer`.
    ///
    /// # Example
    /// ```rust
    /// # use rustica_keys::ssh::Writer;
    /// 
    /// let mut writer = Writer::new();
    /// writer.write_string("some data");
    /// let bytes = writer.into_bytes();
    /// assert_eq!(bytes, [0, 0, 0, 9, 115, 111, 109, 101, 32, 100, 97, 116, 97]);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_slice()
    }
}

impl Default for Writer {
    fn default() -> Self {
        Writer::new()
    }
}