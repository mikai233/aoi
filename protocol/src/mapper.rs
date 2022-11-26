use std::any::Any;
use std::collections::HashMap;

use anyhow::anyhow;
use lazy_static::lazy_static;
use protobuf::{MessageDyn, MessageFull};
use protobuf::reflect::{FieldDescriptor, MessageDescriptor, RuntimeType};
use tokio_kcp::KcpConfig;

use crate::cs_msg::CS_MSG;
use crate::sc_msg::SC_MSG;

lazy_static! {
    pub static ref CS_ID_DESC_MAP: HashMap<i32, MessageDescriptor> = id_to_descriptor(CS_MSG::descriptor()).unwrap() ;
    pub static ref CS_NAME_ID_MAP: HashMap<String, i32> = name_to_id(CS_MSG::descriptor()).unwrap() ;
    pub static ref SC_ID_DESC_MAP: HashMap<i32, MessageDescriptor> = id_to_descriptor(SC_MSG::descriptor()).unwrap() ;
    pub static ref SC_NAME_ID_MAP: HashMap<String, i32> = name_to_id(SC_MSG::descriptor()).unwrap() ;
}

pub fn name_to_id(descriptor: MessageDescriptor) -> anyhow::Result<HashMap<String, i32>> {
    let mut m = HashMap::new();
    for field in descriptor.fields() {
        let id = field.number();
        let message_descriptor = get_msg_descriptor_form_msg_field(field)?;
        m.insert(message_descriptor.name().to_string(), id);
    }
    Ok(m)
}

pub fn id_to_descriptor(
    descriptor: MessageDescriptor,
) -> anyhow::Result<HashMap<i32, MessageDescriptor>> {
    let mut m = HashMap::new();
    for field in descriptor.fields() {
        let id = field.number();
        let message_descriptor = get_msg_descriptor_form_msg_field(field)?;
        m.insert(id, message_descriptor);
    }
    Ok(m)
}

pub fn get_msg_descriptor_form_msg_field(
    field: FieldDescriptor,
) -> anyhow::Result<MessageDescriptor> {
    let rt = field.singular_runtime_type();
    match rt {
        RuntimeType::Message(message) => Ok(message),
        _ => Err(anyhow!("{} only support message field", field.name())),
    }
}

pub fn cast<T: Any>(msg: Box<dyn MessageDyn>) -> anyhow::Result<Box<T>> {
    let msg = msg
        .downcast_box::<T>()
        .map_err(|m| anyhow!("cast message:{} failed", m))?;
    Ok(msg)
}

pub fn kcp_config() -> KcpConfig {
    let mut cfg = tokio_kcp::KcpConfig::default();
    // cfg.flush_write = true;
    // cfg.flush_acks_input = true;
    // cfg.stream = true;
    cfg.nodelay = tokio_kcp::KcpNoDelayConfig::fastest();
    cfg
}
