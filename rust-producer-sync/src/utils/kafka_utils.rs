use std::collections::HashMap;
use std::ffi::CStr;
use std::str::Utf8Error;
use rdkafka::ClientConfig;
use rdkafka::error::KafkaError;
use crate::utils::kafka_utils::ConfDumpError::{ByteStrError, NativeConfError};

#[derive(Debug)]
pub(crate) enum ConfDumpError {
    NativeConfError(KafkaError),
    ByteStrError(Utf8Error)
}

impl From<KafkaError> for ConfDumpError {
    fn from(err: KafkaError) -> Self {
        NativeConfError(err)
    }
}

impl From<Utf8Error> for ConfDumpError {
    fn from(err: Utf8Error) -> Self {
        ByteStrError(err)
    }
}

pub(crate) unsafe fn conf_dump(config: &ClientConfig) -> Result<HashMap<String, String>, ConfDumpError> {
    let native_config = config.create_native_config()?;
    let mut size = 0;
    let ptr = rdkafka_sys::rd_kafka_conf_dump(native_config.ptr(), &mut size);
    let mut map = HashMap::with_capacity(size / 2);
    let mut i = 0;
    while i < size - 1 {
        let k_raw = *ptr.add(i);
        i += 1;
        let v_raw = *ptr.add(i);
        let k = CStr::from_ptr(k_raw).to_str()?;
        let v = CStr::from_ptr(v_raw).to_str()?;
        map.insert(k.to_string(), v.to_string());
        i += 1;
    }
    Ok(map)
}