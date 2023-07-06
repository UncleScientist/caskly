pub(crate) trait StreamHandler {
    fn put_char(ch: u8);
    fn put_string(s: &str);
    fn put_buffer(buf: &[u8]);
    fn put_char_uni(ch: char);
    // note: put_string_uni() is not here because put_string() handles it
    fn put_buffer_uni(buf: &[char]);
}
