use crate::client::ClientResult;
use ads_proto::proto::ams_header::AmsHeader;
use ads_proto::proto::command_id::CommandID;
use ads_proto::proto::proto_traits::ReadFrom;
use ads_proto::proto::response::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

type SenderTable = HashMap<u32, Sender<ClientResult<Response>>>;
type SenderTableAdsNotification = HashMap<u32, Sender<ClientResult<AdsNotificationStream>>>;

//Tcp Header size without response data
pub const AMS_TCP_HEADER_SIZE: usize = 6;

pub fn run_reader_thread(
    stream: TcpStream,
    rx_general: Receiver<(u32, Sender<ClientResult<Response>>)>,
    rx_device_notification: Receiver<(u32, Sender<ClientResult<AdsNotificationStream>>)>,
) -> ClientResult<bool> {
    let mut stream = stream.try_clone()?;
    thread::spawn(move || {
        let mut ams_header;
        let mut sender_table_general: SenderTable = HashMap::new();
        let mut sender_table_device_notivication: SenderTableAdsNotification = HashMap::new();

        loop {
            //read tcp data (blocking)
            match read(&mut stream) {
                Ok(h) => ams_header = h,
                Err(_) => {
                    continue; //ToDo error handling (?)
                }
            }

            //get the latest mpsc sender.
            update_sender_table(&rx_general, &mut sender_table_general);
            update_sender_table_device_notification(
                &rx_device_notification,
                &mut sender_table_device_notivication,
            );

            //Send data to client
            forward_data(
                &mut ams_header,
                &mut sender_table_general,
                &mut sender_table_device_notivication,
            );
        }
    });
    Ok(true)
}

fn update_sender_table(
    rx: &Receiver<(u32, Sender<ClientResult<Response>>)>,
    sender_table: &mut HashMap<u32, Sender<ClientResult<Response>>>,
) {
    while let Ok(s) = rx.try_recv() {
        sender_table.insert(s.0, s.1);
    }
}

fn update_sender_table_device_notification(
    rx: &Receiver<(u32, Sender<ClientResult<AdsNotificationStream>>)>,
    sender_table: &mut HashMap<u32, Sender<ClientResult<AdsNotificationStream>>>,
) {
    while let Ok(s) = rx.try_recv() {
        sender_table.insert(s.0, s.1);
    }
}

fn read(tcp_stream: &mut TcpStream) -> ClientResult<AmsHeader> {
    //ToDo update when ads-proto v0.1.1
    let mut buf = vec![0; AMS_TCP_HEADER_SIZE]; //reserved + length
    tcp_stream.read_exact(&mut buf)?;
    let mut slice = buf.as_slice();
    let _ = slice.read_u16::<LittleEndian>(); //first 2 bytes are not needed
    let length = slice.read_u32::<LittleEndian>()?;
    let mut buf: Vec<u8> = vec![0; length as usize];
    tcp_stream.read_exact(&mut buf)?;
    let ams_header = AmsHeader::read_from(&mut buf.as_slice())?;
    Ok(ams_header)
}

fn forward_data(
    ams_header: &mut AmsHeader,
    sender_table_general: &mut SenderTable,
    sender_table_device_notivication: &mut SenderTableAdsNotification,
) {
    match ams_header.command_id() {
        CommandID::DeviceNotification => {
            let ads_notification: AdsNotificationStream = ams_header
                .response()
                .expect("Not possible to extract response from AmsHeader!")
                .try_into()
                .expect("try_into AdsNotificationStream failed!");

            forward_ads_notification(
                sender_table_device_notivication,
                &ams_header.invoke_id(),
                ads_notification,
            );
        }
        _ => {
            forward_response(
                sender_table_general,
                &ams_header.invoke_id(),
                ams_header
                    .response()
                    .expect("Not possible to extract response from AmsHeader!"),
            );
        }
    }
}

fn forward_ads_notification(
    sender_table: &mut SenderTableAdsNotification,
    id: &u32,
    notification: AdsNotificationStream,
) -> bool {
    if sender_table.contains_key(id) {
        if let Some(tx) = sender_table.remove(id) {
            tx.send(Ok(notification)).expect(
                "Failed to send response from reader thread to parent thread by mpsc channel!",
            );
            return true;
        }
    }
    false
}

fn forward_response(sender_table: &mut SenderTable, id: &u32, response: Response) -> bool {
    if sender_table.contains_key(id) {
        if let Some(tx) = sender_table.remove(id) {
            tx.send(Ok(response)).expect(
                "Failed to send response from reader thread to parent thread by mpsc channel!",
            );
            return true;
        }
    }
    false
}
