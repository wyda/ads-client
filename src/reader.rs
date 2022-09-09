use crate::client::ClientResult;
use crate::client::AMS_HEADER_SIZE;
use ads_proto::proto::ams_header::AmsTcpHeader;
use ads_proto::proto::command_id::CommandID;
use ads_proto::proto::proto_traits::ReadFrom;
use ads_proto::proto::response::*;
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

type SenderTable = HashMap<u32, Sender<ClientResult<Response>>>;
type SenderTableAdsNotification = HashMap<u32, Sender<ClientResult<AdsNotificationStream>>>;

pub fn run_reader_thread(
    stream: TcpStream,
    rx_general: Receiver<(u32, Sender<ClientResult<Response>>)>,
    rx_device_notification: Receiver<(u32, Sender<ClientResult<AdsNotificationStream>>)>,
) {
    thread::spawn(move || {
        let mut ams_tcp_header;
        let mut ams_header;
        let mut sender_table_general: SenderTable = HashMap::new();
        let mut sender_table_device_notivication: SenderTableAdsNotification = HashMap::new();

        loop {
            update_sender_table(&rx_general, &mut sender_table_general);
            update_sender_table_device_notification(
                &rx_device_notification,
                &mut sender_table_device_notivication,
            );

            match read(&stream) {
                Ok(h) => ams_tcp_header = h,
                Err(_) => continue, // ToDo
            }

            ams_header = ams_tcp_header.ams_header;
            match ams_header.command_id() {
                CommandID::DeviceNotification => {
                    let ads_notification: AdsNotificationStream = ams_header
                        .response()
                        .expect("Not possible to extract response from AmsHeader!")
                        .try_into()
                        .expect("try_into AdsNotificationStream failed!");

                    forward_ads_notification(
                        &mut sender_table_device_notivication,
                        &ams_header.invoke_id(),
                        ads_notification,
                    );
                }
                _ => {
                    forward_response(
                        &mut sender_table_general,
                        &ams_header.invoke_id(),
                        ams_header
                            .response()
                            .expect("Not possible to extract response from AmsHeader!"),
                    );
                }
            }
        }
    });
}

fn update_sender_table(
    rx: &Receiver<(u32, Sender<ClientResult<Response>>)>,
    sender_table: &mut HashMap<u32, Sender<ClientResult<Response>>>,
) {
    if let Ok(s) = rx.try_recv() {
        sender_table.insert(s.0, s.1);
    }
}

fn update_sender_table_device_notification(
    rx: &Receiver<(u32, Sender<ClientResult<AdsNotificationStream>>)>,
    sender_table: &mut HashMap<u32, Sender<ClientResult<AdsNotificationStream>>>,
) {
    if let Ok(s) = rx.try_recv() {
        sender_table.insert(s.0, s.1);
    }
}

fn read(tcp_stream: &TcpStream) -> ClientResult<AmsTcpHeader> {
    //ToDo update when ads-proto v0.1.1
    let mut buf = vec![0; AMS_HEADER_SIZE];
    let mut reader = BufReader::new(tcp_stream);
    reader.read_exact(&mut buf)?;
    let ams_tcp_header = AmsTcpHeader::read_from(&mut buf.as_slice())?;
    if ams_tcp_header.ams_header.data_len() > 0 {
        let mut buf2 = vec![0; ams_tcp_header.ams_header.data_len() as usize];
        reader.read_exact(&mut buf2)?;
        buf.append(&mut buf2);
        return Ok(AmsTcpHeader::read_from(&mut buf.as_slice())?);
    }
    Ok(ams_tcp_header)
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
