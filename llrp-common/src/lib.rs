
pub trait DecodableMessage: Sized {
    const ID: u16;

    fn decode(data: &[u8]) -> std::io::Result<Self>;
}
