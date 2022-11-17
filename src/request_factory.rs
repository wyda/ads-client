use ads_proto::ads_services::system_services::{
    GET_SYMHANDLE_BY_NAME, READ_WRITE_SYMVAL_BY_HANDLE,
};
use ads_proto::proto::request::*;

pub fn get_var_handle_request(var_name: &str) -> ReadWriteRequest {
    ReadWriteRequest::new(
        GET_SYMHANDLE_BY_NAME.index_group,
        GET_SYMHANDLE_BY_NAME.index_offset_start,
        4,
        var_name.as_bytes().to_vec(),
    )
}

pub fn get_read_request(handle: u32, len: u32) -> ReadRequest {
    ReadRequest::new(READ_WRITE_SYMVAL_BY_HANDLE.index_group, handle, len)
}

pub fn get_write_request(handle: u32, data: Vec<u8>) -> WriteRequest {
    WriteRequest::new(READ_WRITE_SYMVAL_BY_HANDLE.index_group, handle, data)
}
