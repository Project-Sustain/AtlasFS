use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use communication::StreamHandler;
use hdfs_comm::block::BlockInputStream;
use hdfs_protos::hadoop::hdfs::{BlockOpResponseProto, OpReadBlockProto, OpWriteBlockProto, Status};
use prost::Message;

use std::net::TcpStream;
use std::io::{Read, Write};

static PROTOCOL_VERSION: u16 = 28;

pub struct TransferStreamHandler {
}

impl TransferStreamHandler {
    pub fn new() -> TransferStreamHandler {
        TransferStreamHandler {
        }
    }
}

impl StreamHandler for TransferStreamHandler {
    fn process(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        loop {
            // read op
            let version = stream.read_u16::<BigEndian>()?;
            let op_type = stream.read_u8()?;
            
            if version != PROTOCOL_VERSION {
                // TODO - error
            }

            // calculate leb128 encoded op proto length
            let mut length = 0;
            for i in 0..8 {
                let byte = stream.read_u8()?;
                let delta = ((byte << 1 >> 1) as u64) << (i * 7);
                length += delta;

                if byte < 64 {
                    break;
                }
            }

            // read op proto into buffer
            let mut buf = vec![0u8; length as usize];
            stream.read_exact(&mut buf)?;

            // read in proto
            match op_type {
                80 => {
                    // parse write block op
                    let owb_proto = OpWriteBlockProto::decode(&buf);
                    debug!("WriteBlock: {:?}", owb_proto);

                    // send op response
                    let mut bor_proto = BlockOpResponseProto::default();
                    bor_proto.status = Status::Success as i32;

                    let mut resp_buf = Vec::new();
                    bor_proto.encode_length_delimited(&mut resp_buf)?;
                    stream.write_all(&resp_buf)?;

                    // recv block
                    // TODO - parameterize these values
                    let chunk_size_bytes = 512;
                    let chunks_per_packet = 126;

                    let mut block = Vec::new();
                    let mut block_stream = BlockInputStream::new(
                        stream.try_clone().unwrap(),
                        chunk_size_bytes, chunks_per_packet);
                    block_stream.read_to_end(&mut block);
                    block_stream.close();

                    debug!("read {} bytes into block", block.len());
                },
                81 => {
                    // parse read block op
                    let orb_proto = OpReadBlockProto::decode(&buf);
                    debug!("ReadBlock: {:?}", orb_proto);
 
                    // TODO - send op respone
 
                    // TODO - send block
                    unimplemented!();
                },
                _ => unimplemented!(),
            }
        }
    }
}