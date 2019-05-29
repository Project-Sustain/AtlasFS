use hdfs_comm::rpc::Protocol;
use prost::Message;
use shared::NahError;
use shared::protos::{IndexReportResponseProto, IndexReportRequestProto};

use crate::index::Index;

use std::sync::{Arc, RwLock};

pub struct NahfsProtocol {
    index: Arc<RwLock<Index>>,
}

impl NahfsProtocol {
    pub fn new(index: Arc<RwLock<Index>>) -> NahfsProtocol {
        NahfsProtocol {
            index: index,
        }
    }

    fn index_report(&self, req_buf: &[u8],
            resp_buf: &mut Vec<u8>) -> Result<(), NahError> {
        let request = IndexReportRequestProto
            ::decode_length_delimited(req_buf)?;
        let response = IndexReportResponseProto::default();

        // process index report
        trace!("indexReport({:?})", request);
        let mut index = self.index.write().unwrap();
        for i in 0..request.block_ids.len() {
            let block_id = &request.block_ids[i];
            let block_index = &request.block_indices[i];

            for j in 0..block_index.geohashes.len() {
                index.add_geohash(&block_index.geohashes[j], block_id,
                    block_index.end_indices[j]
                        - block_index.start_indices[j])?;
            }
        }

        response.encode_length_delimited(resp_buf)?;
        Ok(())
    }
}

impl Protocol for NahfsProtocol {
    fn process(&self, _user: &Option<String>, method: &str,
            req_buf: &[u8], resp_buf: &mut Vec<u8>) -> std::io::Result<()> {
        match method {
            "indexReport" => self.index_report(req_buf, resp_buf)?,
            _ => error!("unimplemented method '{}'", method),
        }

        Ok(())
    }
}
