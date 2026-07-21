//! Minimal protobuf wire decoder for Bilibili live WS V2 payloads.
//!
//! Only implements the subset we need (varint / length-delimited / fixed32/64).
//! Field semantics come from community-reversed `.proto` definitions.

use std::collections::HashMap;

/// A decoded protobuf field value.
#[derive(Debug, Clone)]
pub enum PbValue {
    Varint(u64),
    Fixed64(u64),
    Bytes(Vec<u8>),
    Fixed32(u32),
}

impl PbValue {
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Varint(v) => Some(*v),
            Self::Fixed64(v) => Some(*v),
            Self::Fixed32(v) => Some(*v as u64),
            Self::Bytes(_) => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.as_u64().map(|v| v as i64)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Bytes(b) => std::str::from_utf8(b).ok(),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            _ => None,
        }
    }
}

/// Decoded message: field number → one or more values (repeated fields).
#[derive(Debug, Clone, Default)]
pub struct PbMessage {
    fields: HashMap<u32, Vec<PbValue>>,
}

impl PbMessage {
    pub fn get_first(&self, field: u32) -> Option<&PbValue> {
        self.fields.get(&field).and_then(|v| v.first())
    }

    pub fn get_all(&self, field: u32) -> &[PbValue] {
        self.fields.get(&field).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn get_u64(&self, field: u32) -> Option<u64> {
        self.get_first(field)?.as_u64()
    }

    pub fn get_i64(&self, field: u32) -> Option<i64> {
        self.get_first(field)?.as_i64()
    }

    pub fn get_str(&self, field: u32) -> Option<&str> {
        self.get_first(field)?.as_str()
    }

    pub fn get_message(&self, field: u32) -> Option<PbMessage> {
        let bytes = self.get_first(field)?.as_bytes()?;
        decode_message(bytes).ok()
    }

    pub fn get_messages(&self, field: u32) -> Vec<PbMessage> {
        self.get_all(field)
            .iter()
            .filter_map(|v| v.as_bytes())
            .filter_map(|b| decode_message(b).ok())
            .collect()
    }
}

/// Decode a protobuf message from bytes.
pub fn decode_message(buf: &[u8]) -> Result<PbMessage, String> {
    let mut fields: HashMap<u32, Vec<PbValue>> = HashMap::new();
    let mut i = 0;

    while i < buf.len() {
        let (key, next) = read_varint(buf, i)?;
        i = next;
        let field = (key >> 3) as u32;
        let wire = (key & 0x7) as u8;

        let value = match wire {
            0 => {
                let (v, next) = read_varint(buf, i)?;
                i = next;
                PbValue::Varint(v)
            }
            1 => {
                if i + 8 > buf.len() {
                    return Err("truncated fixed64 truncated".into());
                }
                let v = u64::from_le_bytes(buf[i..i + 8].try_into().unwrap());
                i += 8;
                PbValue::Fixed64(v)
            }
            2 => {
                let (len, next) = read_varint(buf, i)?;
                i = next;
                let len = len as usize;
                if i + len > buf.len() {
                    return Err("protobuf bytes truncated".into());
                }
                let bytes = buf[i..i + len].to_vec();
                i += len;
                PbValue::Bytes(bytes)
            }
            5 => {
                if i + 4 > buf.len() {
                    return Err("protobuf fixed32 truncated".into());
                }
                let v = u32::from_le_bytes(buf[i..i + 4].try_into().unwrap());
                i += 4;
                PbValue::Fixed32(v)
            }
            other => {
                return Err(format!("unsupported protobuf wire type {other}"));
            }
        };

        fields.entry(field).or_default().push(value);
    }

    Ok(PbMessage { fields })
}

fn read_varint(buf: &[u8], mut i: usize) -> Result<(u64, usize), String> {
    let mut result: u64 = 0;
    let mut shift = 0;
    loop {
        if i >= buf.len() {
            return Err("protobuf varint truncated".into());
        }
        let byte = buf[i];
        i += 1;
        result |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok((result, i));
        }
        shift += 7;
        if shift >= 64 {
            return Err("protobuf varint overflow".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_simple_string_and_varint() {
        // field 1 varint 150, field 2 string "testing"
        // 08 96 01 12 07 74 65 73 74 69 6e 67
        let buf = [
            0x08, 0x96, 0x01, 0x12, 0x07, b't', b'e', b's', b't', b'i', b'n', b'g',
        ];
        let msg = decode_message(&buf).unwrap();
        assert_eq!(msg.get_u64(1), Some(150));
        assert_eq!(msg.get_str(2), Some("testing"));
    }
}
