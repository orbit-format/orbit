use crate::value::OrbitValue;

pub fn to_msgpack_bytes(value: &OrbitValue) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(value)
}
