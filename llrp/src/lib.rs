mod binary;

#[cfg(test)]
mod tests;

pub use crate::binary::{read_message, write_message, BinaryMessage};

include!(concat!(env!("OUT_DIR"), "/llrp_generated.rs"));
