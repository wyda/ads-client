use crate::client::ClientResult;
use ads_proto::error::AdsError;
use ads_proto::proto::ams_header::AmsHeader;
use ads_proto::proto::command_id::CommandID;
use ads_proto::proto::proto_traits::ReadFrom;
use ads_proto::proto::response::*;
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::{ErrorKind, Read};
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
    rx_update_tcp_stream: Receiver<TcpStream>,
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
                Err(e) => {
                    match e.kind() {
                        ErrorKind::UnexpectedEof => {
                            //get the latest mpsc sender.
                            update_sender_table(&rx_general, &mut sender_table_general);
                            update_sender_table_device_notification(
                                &rx_device_notification,
                                &mut sender_table_device_notivication,
                            );
                            notify_connection_down(
                                &mut sender_table_general,
                                &mut sender_table_device_notivication,
                            );
                            //Update TCP Stream
                            stream = update_tcp_stream(&rx_update_tcp_stream, stream);
                            continue;
                        }
                        _ => {
                            //Update TCP Stream
                            stream = update_tcp_stream(&rx_update_tcp_stream, stream);
                            continue;
                        }
                    }
                }
            }
            //Update TCP Stream
            stream = update_tcp_stream(&rx_update_tcp_stream, stream);
            //get the latest mpsc sender.
            update_sender_table(&rx_general, &mut sender_table_general);
            update_sender_table_device_notification(
                &rx_device_notification,
                &mut sender_table_device_notivication,
            );

            //Send data to client
            match ams_header.ads_error() {
                AdsError::ErrNoError => {
                    forward_data(
                        &mut ams_header,
                        &mut sender_table_general,
                        &mut sender_table_device_notivication,
                    );
                }
                AdsError::ErrPortNotConnected => notify_connection_down(
                    &mut sender_table_general,
                    &mut sender_table_device_notivication,
                ),
                _ => continue,
            };
            println!("{:?}", ams_header.ads_error());
        }
    });
    Ok(true)
}

fn update_tcp_stream(rx_tcp_stream: &Receiver<TcpStream>, stream: TcpStream) -> TcpStream {
    if let Ok(s) = rx_tcp_stream.try_recv() {        
        return s;
    } else {
        return stream;
    }
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

fn read(tcp_stream: &mut TcpStream) -> Result<AmsHeader, std::io::Error> {
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

            for header in &ads_notification.ads_stamp_headers {
                for sample in &header.notification_samples {
                    forward_ads_notification(
                        sender_table_device_notivication,
                        &sample.notification_handle,
                        ads_notification.clone(),
                    );
                }
            }
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
        if let Some(tx) = sender_table.get(id) {
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

fn notify_connection_down(
    sender_table: &mut SenderTable,
    sender_table_device_notivication: &mut SenderTableAdsNotification,
) {
    let mut delete_list = Vec::new();
    for (id, tx) in sender_table.clone() {
        if tx
            .send(Err(anyhow!(AdsError::ErrPortNotConnected)))
            .is_err()
        {
            delete_list.push(id);
        }
    }

    for id in &delete_list {
        sender_table.remove(id);
    }

    let mut delete_notification_list = Vec::new();
    for (id, tx) in sender_table_device_notivication.clone() {
        if tx
            .send(Err(anyhow!(AdsError::ErrPortNotConnected)))
            .is_err()
        {
            delete_notification_list.push(id);
        }
    }

    for id in &delete_notification_list {
        sender_table_device_notivication.remove(id);
    }
}
