use ads_proto::ads_services::system_services::{
    ADSIGRP_SUMUP_READEX, ADSIGRP_SUMUP_WRITE, GET_SYMHANDLE_BY_NAME, READ_WRITE_SYMVAL_BY_HANDLE,
    RELEASE_SYMHANDLE, GET_ETHERCAT_MASTER_STATE, GET_PROJECTED_SLAVES, GET_FIXED_ADDRESSES, GET_SLAVE_STATUS
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

pub fn get_sumup_read_request(
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

pub fn get_sumup_write_request(
    index_offset: u32,
    read_len: u32,
    write_data: Vec<u8>,
) -> ReadWriteRequest {
    ReadWriteRequest::new(
        ADSIGRP_SUMUP_WRITE.index_group,
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

/// Mater state as u16
/// 0x0000: Init State
/// 0x0002: Pre-Operational State
/// 0x0003: Bootstrap State
/// 0x0004: Safe-Operational State
/// 0x0008: Operational State
pub fn get_ethercat_master_state() -> ReadRequest {
    ReadRequest::new(GET_ETHERCAT_MASTER_STATE.index_group, GET_ETHERCAT_MASTER_STATE.index_offset_start, 2)
}

/// Number of slaves as u16
pub fn get_projected_slaves() -> ReadRequest {
    ReadRequest::new(GET_PROJECTED_SLAVES.index_group, GET_PROJECTED_SLAVES.index_offset_start, 2)
}

/// Returns the fixed addresses of all slaves. 
/// len --> Read length in bytes
/// use get_projected_slaves() to get the len
pub fn get_fixed_addresses(len: u32) -> ReadRequest {
    ReadRequest::new(GET_FIXED_ADDRESSES.index_group, GET_FIXED_ADDRESSES.index_offset_start, len)
}

///Returns the EtherCAT status and the Link status of all Slaves:
///Use get_projected_slaves() to get the number of slaves to calculate the len param.
/// {
///BYTE
///EtherCAT state of a slave. The state can adopt one of the following values:
///0x0000: Init State
///0x0002: Pre-Operational State
///0x0003: Bootstrap State
///0x0004: Safe-Operational State
///0x0008: Operational State

///Additionally following bits can be set:
///0x0010: Error State
///0x0020: Invalid VPRS( VendorId, Product Code, RevisionsNo or SerialNo)
///0x0040: Initialization command error

///BYTE
///Link status of an EtherCAT slave. The Link status can consist of an ORing of the following bits:
///0x0000: Link ok.
///0x0001: Link not present
///0x0002: No communication
///0x0004: Link missing
///0x0008: Additional link
///0x0010: Port A
///0x0020: Port B
///0x0040: Port C
///0x0080: Port D

///example: 0x0024 = Missing Link at port B.
///}[nSlaves]
pub fn get_slave_status(len: u32) -> ReadRequest {
    ReadRequest::new(GET_SLAVE_STATUS.index_group, GET_SLAVE_STATUS.index_offset_start, len)
}