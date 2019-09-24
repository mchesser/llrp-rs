pub mod deserializer;

#[cfg(test)]
mod tests;

include!(concat!(env!("OUT_DIR"), "/llrp_generated.rs"));
