use std::fmt::{Display, Formatter};
use std::io;

use anyhow::anyhow;
use bytes::{BufMut, BytesMut};
use futures::stream::{SplitSink, SplitStream};
use protobuf::MessageDyn;
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::mapper::{CS_ID_DESC_MAP, CS_NAME_ID_MAP, SC_ID_DESC_MAP, SC_NAME_ID_MAP};

pub type MessageSink = SplitSink<Framed<TcpStream, ProtoCodec>, Box<dyn MessageDyn>>;
pub type MessageStream = SplitStream<Framed<TcpStream, ProtoCodec>>;

pub struct ProtoCodec {
    pub is_server: bool,
}

impl ProtoCodec {
    pub fn new(is_server: bool) -> Self {
        Self { is_server }
    }

    pub fn parse_proto(&self, id: i32, msg_bytes: Vec<u8>) -> anyhow::Result<Box<dyn MessageDyn>> {
        let descriptor = if self.is_server {
            CS_ID_DESC_MAP
                .get(&id)
                .ok_or(anyhow!("id:{} not found in sc", id))?
        } else {
            SC_ID_DESC_MAP
                .get(&id)
                .ok_or(anyhow!("id:{} not found in cs", id))?
        };
        let msg = descriptor.parse_from_bytes(&*msg_bytes)?;
        Ok(msg)
    }

    pub fn get_proto_id(&self, msg: &Box<dyn MessageDyn>) -> anyhow::Result<i32> {
        let desc = msg.descriptor_dyn();
        let msg_name = desc.name();
        let id = if self.is_server {
            SC_NAME_ID_MAP
                .get(msg_name)
                .ok_or(anyhow!("msg:{} not found in sc", msg_name))?
        } else {
            CS_NAME_ID_MAP
                .get(msg_name)
                .ok_or(anyhow!("msg:{} not found in cs", msg_name))?
        };
        Ok(*id)
    }
}

impl Decoder for ProtoCodec {
    type Item = Box<dyn MessageDyn>;
    type Error = ProtoCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let buf_len = src.len();
        if buf_len < 2 {
            return Ok(None);
        }
        let mut package_len_bytes = [0u8; 2];
        package_len_bytes.copy_from_slice(&src[..2]);
        let package_len = u16::from_be_bytes(package_len_bytes) as usize;
        return if buf_len < package_len {
            src.reserve(package_len - buf_len);
            Ok(None)
        } else {
            let src = src.split_to(package_len);
            let mut id_bytes = [0u8; 2];
            id_bytes.copy_from_slice(&src[2..4]);
            let id = u16::from_be_bytes(id_bytes) as i32;
            let mut msg_bytes = vec![0u8; package_len - 4];
            msg_bytes.copy_from_slice(&src[4..package_len]);
            let msg = self.parse_proto(id, msg_bytes)?;
            Ok(Some(msg))
        };
    }
}

impl Encoder<Box<dyn MessageDyn>> for ProtoCodec {
    type Error = ProtoCodecError;

    fn encode(&mut self, msg: Box<dyn MessageDyn>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let id = self.get_proto_id(&msg)?;
        let body = msg.write_to_bytes_dyn()?;
        let package_len = 2 + 2 + body.len();
        dst.put_u16(u16::try_from(package_len)?);
        dst.put_u16(u16::try_from(id)?);
        dst.put_slice(body.as_slice());
        Ok(())
    }
}

#[derive(Debug)]
pub enum ProtoCodecError {
    Protobuf(protobuf::Error),
    Anyhow(anyhow::Error),
    Io(io::Error),
    TryFromInt(std::num::TryFromIntError),
}

impl Display for ProtoCodecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtoCodecError::Io(e) => {
                write!(f, "{}", e)
            }
            ProtoCodecError::Protobuf(e) => {
                write!(f, "{}", e)
            }
            ProtoCodecError::Anyhow(e) => {
                write!(f, "{}", e)
            }
            ProtoCodecError::TryFromInt(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

impl From<io::Error> for ProtoCodecError {
    fn from(e: io::Error) -> ProtoCodecError {
        ProtoCodecError::Io(e)
    }
}

impl From<anyhow::Error> for ProtoCodecError {
    fn from(value: anyhow::Error) -> Self {
        ProtoCodecError::Anyhow(value)
    }
}

impl From<protobuf::Error> for ProtoCodecError {
    fn from(value: protobuf::Error) -> Self {
        ProtoCodecError::Protobuf(value)
    }
}

impl From<std::num::TryFromIntError> for ProtoCodecError {
    fn from(value: std::num::TryFromIntError) -> Self {
        ProtoCodecError::TryFromInt(value)
    }
}

impl std::error::Error for ProtoCodecError {}
