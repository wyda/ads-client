use ads_proto::ads_services::system_services::{
    ADSIGRP_SUMUP_READEX, GET_SYMHANDLE_BY_NAME, READ_WRITE_SYMVAL_BY_HANDLE, RELEASE_SYMHANDLE,
};
use ads_proto::proto::ads_state::AdsState;
use ads_proto::proto::ads_transition_mode::AdsTransMode;
use ads_proto::proto::request::*;

pub fn get_var_handle_request(var_name: &str) -> ReadWriteRequest {
    ReadWriteRequest::new(
        GET_SYMHANDLE_BY_NAME.index_group,
        GET_SYMHANDLE_BY_NAME.index_offset_start,
        4,
        var_name.as_bytes().to_vec(),
    )
}

pub fn get_release_handle_request(handle: u32) -> WriteRequest {
    WriteRequest::new(
        RELEASE_SYMHANDLE.index_group,
        RELEASE_SYMHANDLE.index_offset_start,
        handle.to_le_bytes().to_vec(),
    )
}

pub fn get_read_request(handle: u32, len: u32) -> ReadRequest {
    ReadRequest::new(READ_WRITE_SYMVAL_BY_HANDLE.index_group, handle, len)
}

pub fn get_write_request(handle: u32, data: Vec<u8>) -> WriteRequest {
    WriteRequest::new(READ_WRITE_SYMVAL_BY_HANDLE.index_group, handle, data)
}

pub fn get_write_control_request(ads_state: AdsState, device_state: u16) -> WriteControlRequest {
    let data: Vec<u8> = Vec::with_capacity(0);
    WriteControlRequest::new(ads_state, device_state, 0, data)
}

pub fn get_read_write_request(
    index_offset: u32,
    read_len: u32,
    write_data: Vec<u8>,
) -> ReadWriteRequest {
    ReadWriteRequest::new(
        GET_SYMHANDLE_BY_NAME.index_group,
        index_offset,
        read_len,
        write_data,
    )
}

pub fn get_sumup_read_write_request(
    index_offset: u32,
    read_len: u32,
    write_data: Vec<u8>,
) -> ReadWriteRequest {
    ReadWriteRequest::new(
        ADSIGRP_SUMUP_READEX.index_group,
        index_offset,
        read_len,
        write_data,
    )
}

pub fn get_add_device_notification(
    handle: u32,
    length: u32,
    transmission_mode: AdsTransMode,
    max_delay: u32,
    cycle_time: u32,
) -> AddDeviceNotificationRequest {
    AddDeviceNotificationRequest::new(
        READ_WRITE_SYMVAL_BY_HANDLE.index_group,
        handle,
        length,
        transmission_mode,
        max_delay,
        cycle_time,
    )
}

pub fn get_delete_device_notification(handle: u32) -> DeleteDeviceNotificationRequest {
    DeleteDeviceNotificationRequest::new(handle)
}
