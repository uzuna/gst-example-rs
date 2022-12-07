//! Ecample metaklv impl
use ers_meta::{ExampleRsMeta, ExampleRsMetaParams};
use klv::{value::Value, DataSet, ParseError};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExampleDataset {
    Index = 2,
    Mode = 3,
    Label = 10,
}

impl ExampleDataset {
    const KEY: &str = "gstexamplers0000";
}

impl TryFrom<u8> for ExampleDataset {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use ExampleDataset::*;
        match value {
            x if x == Index as u8 => Ok(Index),
            x if x == Mode as u8 => Ok(Mode),
            x if x == Label as u8 => Ok(Label),
            _ => Err(()),
        }
    }
}

impl DataSet for ExampleDataset {
    type Item = Value;

    fn key() -> &'static [u8] {
        Self::KEY.as_bytes()
    }

    fn from_byte(b: u8) -> Option<Self>
    where
        Self: std::marker::Sized,
    {
        if let Ok(x) = Self::try_from(b) {
            Some(x)
        } else {
            None
        }
    }

    fn value(&self, v: &[u8]) -> Result<Self::Item, ParseError> {
        use ExampleDataset::*;
        match self {
            Index => Ok(Value::as_i32(v)),
            Mode => Ok(Value::as_u16(v)),
            Label => Ok(Value::as_string(v)),
        }
    }

    fn as_byte(&self) -> u8 {
        *self as u8
    }
}

#[allow(dead_code)]
pub(crate) fn encode_klv(meta: &ExampleRsMeta) -> Vec<(ExampleDataset, Value)> {
    vec![
        (ExampleDataset::Mode, Value::U16(meta.mode() as u16)),
        (
            ExampleDataset::Label,
            Value::String(meta.label().to_string()),
        ),
        (ExampleDataset::Index, Value::I32(meta.index())),
    ]
}

pub(crate) fn encode_klv_params(params: &ExampleRsMetaParams) -> Vec<(ExampleDataset, Value)> {
    vec![
        (ExampleDataset::Mode, Value::U16(params.mode as u16)),
        (
            ExampleDataset::Label,
            Value::String(params.label.to_string()),
        ),
        (ExampleDataset::Index, Value::I32(params.index)),
    ]
}

#[cfg(test)]
mod tests {
    use klv::{encode, encode_len, KLVGlobal, KLVReader};

    use super::{ExampleDataset, Value};

    #[test]
    fn test_encode() {
        let records = [
            (ExampleDataset::Mode, Value::U16(4545)),
            (
                ExampleDataset::Label,
                Value::String("asdasdasd".to_string()),
            ),
            (ExampleDataset::Index, Value::I32(1234)),
        ];
        let encode_size = encode_len(&records);
        let mut buf = vec![0_u8; encode_size];
        let write_size = encode(&mut buf, &records).unwrap();
        assert_eq!(encode_size, write_size);

        if let Ok(klvg) = KLVGlobal::try_from_bytes(&buf) {
            let r = KLVReader::<ExampleDataset>::from_bytes(klvg.content());
            for x in r {
                let key = x.key().unwrap();
                assert!(
                    key == ExampleDataset::Mode
                        || key == ExampleDataset::Label
                        || key == ExampleDataset::Index
                );
            }
        } else {
            unreachable!("unknown data {:?}", &buf);
        }
    }
}
