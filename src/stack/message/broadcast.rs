use lz4_flex::block::{compress_prepend_size, decompress_size_prepended};
use std::net::Ipv4Addr;
use crate::stack::Frame;
use crate::stack::frame::{FrameHeader, ToFromFrame};
use crate::stack::util::{parse_bool, parse_ipv4, parse_byte};
use crate::message::MessageType;
use lz4::{Decoder, EncoderBuilder};


/// Broadcast this node to nearby devices.
#[derive(Clone)]
pub struct BroadcastMessage {
    pub header: Option<FrameHeader>,
    pub isgateway: bool,
    pub ipOffset: usize,
    pub ipaddr: Option<Ipv4Addr>
}

impl ToFromFrame for BroadcastMessage {
    fn from_frame(f: &mut Frame) -> std::io::Result<Box<Self>> {
        let header = f.header();
        let data = f.payload();
        let isgateway = parse_bool(data[0]).unwrap();
        let offset = data[1] as usize;
        let mut ipaddr: Option<Ipv4Addr> = None;
        if offset > 0 as usize {
            let octets = &data[2..6];
            ipaddr = Some(parse_ipv4(octets));
        }

        Ok(Box::new(BroadcastMessage {
            header: Some(header),
            isgateway,
            ipOffset: offset,
            ipaddr
        }))
    }

    fn to_frame(&self, frameid: u8, sender: u8, route: Vec<u8>) -> Frame {
        // write the payload
        let mut payload: Vec<u8> = Vec::new();
        payload.push(parse_byte(self.isgateway));

        // write offset and octets if ip assigned
        if self.ipaddr.is_some() {
            payload.push(4usize as u8);
            let ip = self.ipaddr.unwrap();
            let octets = ip.octets();
            octets.iter().for_each(|oct| payload.push(oct.clone()));
        } else {
            payload.push(0usize as u8);
        }

        // cast the route
        let route: Vec<u8> = route.clone().iter().map(|i| i.clone() as u8).collect();
        let routeoffset = route.len() as u8;

        Frame::new(
            0i8 as u8,
            frameid,
            MessageType::Broadcast as u8,
            sender as u8,
            routeoffset as u8,
            route,
            payload
        )
    }
}

#[cfg(test)]
#[test]
fn broadcast_tofrom_frame() {
    let id = 5u8;
    let isgateway = false;
    let msg = BroadcastMessage {
        header: None,
        isgateway,
        ipOffset: 4,
        ipaddr: Some(Ipv4Addr::new(172,16,0,id.clone() as u8))
    };
    let mut route: Vec<u8> = Vec::new();
    route.push(id.clone());

    // check tofrom frame
    let mut frame = msg.to_frame(1u8, id, route);

    assert_eq!(frame.sender(), id);
    assert_eq!(frame.payload().get(0).unwrap().clone() as i8, 0i8);
    assert_eq!(frame.payload().get(1).unwrap().clone() as usize, 4);
    assert_eq!(frame.payload().get(2).unwrap().clone(), 172u8);
    assert_eq!(frame.payload().get(3).unwrap().clone(), 16u8);
    assert_eq!(frame.payload().get(4).unwrap().clone(), 0u8);
    assert_eq!(frame.payload().get(5).unwrap().clone(), id);

    // ensure representation is same after hex encoding
    let bytes = frame.to_bytes();

    assert_eq!(bytes.get(6).unwrap().clone() as i8, 0i8);
    assert_eq!(bytes.get(7).unwrap().clone() as usize, 4);
    assert_eq!(bytes.get(8).unwrap().clone(), 172u8);
    assert_eq!(bytes.get(9).unwrap().clone(), 16u8);
    assert_eq!(bytes.get(10).unwrap().clone(), 0u8);
    assert_eq!(bytes.get(11).unwrap().clone(), id);

    let mut frame2 = Frame::from_bytes(&bytes).unwrap();
    let msg2 = BroadcastMessage::from_frame(&mut frame2).unwrap();

    assert_eq!(frame2.sender(), id);
    assert_eq!(msg2.header.unwrap().sender(), id);
    assert_eq!(msg2.isgateway, isgateway);
    assert_eq!(msg2.ipaddr.unwrap(), msg.ipaddr.unwrap());
}

#[test]
fn lz4test(){
    let input: &[u8] = b"Hello people, what's up?";
    let compressed = compress_prepend_size(input);
    let uncompressed = decompress_size_prepended(&compressed).unwrap();
    assert_eq!(input, uncompressed);
}
