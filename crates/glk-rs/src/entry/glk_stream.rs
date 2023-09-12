use std::{cell::RefCell, rc::Rc};

use crate::{
    file_stream::{FileStream, GlkFileRef},
    mem_stream::MemStream,
    prelude::GlkRock,
    stream::{GlkStreamID, GlkStreamResult},
    windows::GlkWindow,
    Glk, GlkFileMode, GlkSeekMode,
};

impl<T: GlkWindow + Default> Glk<T> {
    /*
     * Section 5 - Streams
     */

    /// set the current stream, or None to disable
    pub fn stream_set_current(&mut self, streamid: GlkStreamID) {
        self.default_stream = Some(streamid)
    }

    /// get the current stream, or None if no stream is set
    pub fn stream_get_current(&self) -> Option<GlkStreamID> {
        self.default_stream
    }

    /*
     * Section 5.1. How to Print
     */

    /// write a byte to the default stream
    pub fn put_char(&mut self, ch: u8) {
        if let Some(stream) = self.default_stream {
            self.put_char_stream(stream, ch);
        }
    }

    /// write a string to the default stream
    pub fn put_string(&mut self, s: &str) {
        if let Some(stream) = self.default_stream {
            self.put_string_stream(stream, s);
        }
    }

    /// write a string to the default stream
    pub fn put_string_uni(&mut self, s: &str) {
        self.put_string(s);
    }

    /// write a byte buffer to the default stream
    pub fn put_buffer(&mut self, buf: &[u8]) {
        if let Some(stream) = self.default_stream {
            self.put_buffer_stream(stream, buf);
        }
    }

    /// write a unicode character to the default stream
    pub fn put_char_uni(&mut self, ch: char) {
        if let Some(stream) = self.default_stream {
            self.put_char_stream_uni(stream, ch);
        }
    }

    /// write a unicode buffer to the default stream
    pub fn put_buffer_uni(&mut self, buf: &[char]) {
        if let Some(stream) = self.default_stream {
            self.put_buffer_stream_uni(stream, buf);
        }
    }

    /// write a byte to a stream
    pub fn put_char_stream(&mut self, streamid: GlkStreamID, ch: u8) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_char(ch);
            if let Some(echo) = stream.get_echo_stream() {
                self.put_char_stream(echo, ch);
            }
        }
    }

    /// write a unicode string to a stream
    pub fn put_string_stream(&mut self, streamid: GlkStreamID, s: &str) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_string(s);
            if let Some(echo) = stream.get_echo_stream() {
                self.put_string_stream(echo, s);
            }
        }
    }

    /// write a unicode string to a stream - same as put_string_stream() in rust because
    /// all strings are unicode in rust
    pub fn put_string_stream_uni(&mut self, streamid: GlkStreamID, s: &str) {
        self.put_string_stream(streamid, s);
    }

    /// write a buffer of bytes to a stream
    pub fn put_buffer_stream(&mut self, streamid: GlkStreamID, buf: &[u8]) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_buffer(buf);
            if let Some(echo) = stream.get_echo_stream() {
                self.put_buffer_stream(echo, buf);
            }
        }
    }

    /// write a unicode character to a stream
    pub fn put_char_stream_uni(&mut self, streamid: GlkStreamID, ch: char) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_char_uni(ch);
            if let Some(echo) = stream.get_echo_stream() {
                self.put_char_stream_uni(echo, ch);
            }
        }
    }

    /// write a buffer of unicode characters to a stream
    pub fn put_buffer_stream_uni(&mut self, streamid: GlkStreamID, buf: &[char]) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_buffer_uni(buf);
            if let Some(echo) = stream.get_echo_stream() {
                self.put_buffer_stream_uni(echo, buf);
            }
        }
    }

    /*
     * Section 5.2 - How to Read
     */

    /// read a byte from a stream. If the stream is output-only, or if there are no
    /// more characters to read, return None.
    pub fn get_char_stream(&mut self, streamid: GlkStreamID) -> Option<u8> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_char()
        } else {
            None
        }
    }

    /// read a stream of bytes
    pub fn get_buffer_stream(&mut self, streamid: GlkStreamID, len: Option<usize>) -> Vec<u8> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_buffer(len)
        } else {
            Vec::new()
        }
    }

    /// read a stream of bytes until a newline, or until end-of-stream
    pub fn get_line_stream(&mut self, streamid: GlkStreamID, len: Option<usize>) -> Vec<u8> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_line(len)
        } else {
            Vec::new()
        }
    }

    /// get a unicode character from a stream. If the stream is output-only, or if there
    /// are no more characters to read, return None
    pub fn get_char_stream_uni(&mut self, streamid: GlkStreamID) -> Option<char> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_char_uni()
        } else {
            None
        }
    }

    /// read a stream of unicode characters
    pub fn get_buffer_stream_uni(&mut self, streamid: GlkStreamID, len: Option<usize>) -> String {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_buffer_uni(len)
        } else {
            String::new()
        }
    }

    /// read a stream of unicode characters
    pub fn get_line_stream_uni(&mut self, streamid: GlkStreamID, len: Option<usize>) -> String {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_line_uni(len)
        } else {
            String::new()
        }
    }

    /*
     * Glk Section 5.3 - Closing Streams
     */

    /// Closes a stream. Window streams are only close-able through glk.window_close()
    pub fn stream_close(
        &mut self,
        streamid: GlkStreamID,
    ) -> Option<(GlkStreamResult, Option<Vec<u8>>)> {
        let stream = self.stream_mgr.get(streamid)?;
        if stream.is_window_stream() {
            return None;
        }

        let result = if stream.is_memory_stream() {
            let result = stream.get_data();
            Some((self.stream_mgr.close(streamid)?, Some(result)))
        } else {
            Some((self.stream_mgr.close(streamid)?, None))
        };

        for win in self.window_iterate() {
            if let Some(mut window) = self.win_mgr.get_ref(win) {
                window.remove_echo_stream_if_matches(streamid);
            }
        }

        result
    }

    /*
     * Glk Section 5.4 - Stream Positions
     */

    /// Get the position within a stream. Return value is offset from the beginning of the stream
    pub fn stream_get_position(&mut self, streamid: GlkStreamID) -> Option<u32> {
        let stream = self.stream_mgr.get(streamid)?;
        Some(stream.get_position())
    }

    /// Sets the position of the next read/write location in the stream
    pub fn stream_set_position(
        &mut self,
        streamid: GlkStreamID,
        pos: i32,
        mode: GlkSeekMode,
    ) -> Option<()> {
        let stream = self.stream_mgr.get(streamid)?;
        stream.set_position(pos, mode)
    }

    /*
     * Glk Section 5.6.2 - Memory Streams
     */

    /// Open a memory-based buffer to do stream I/O
    /// TODO: for read-only streams, we should not have to pass in a mut ref
    pub fn stream_open_memory(
        &mut self,
        buf: Vec<u8>,
        file_mode: GlkFileMode,
        _rock: GlkRock,
    ) -> GlkStreamID {
        let mem_stream = Rc::new(RefCell::new(MemStream::new(buf)));
        self.stream_mgr.new_stream(mem_stream, file_mode)
    }

    /*
     * Glk Section 5.6.3 - File Streams
     */

    /// open a file stream for reading or writing, or both
    pub fn stream_open_file(
        &mut self,
        filerefid: GlkFileRef,
        mode: GlkFileMode,
        rock: GlkRock,
    ) -> Option<GlkStreamID> {
        let fileref = self.fileref_mgr.get(filerefid)?;

        let file_stream = if fileref.is_temp {
            Rc::new(RefCell::new(FileStream::create_temp(fileref, rock)?))
        } else {
            Rc::new(RefCell::new(FileStream::open_file(fileref, mode, rock)?))
        };

        Some(self.stream_mgr.new_stream(file_stream, mode))
    }

    /// open a file stream using unicode encoding. If opening in text mode, the file
    /// is assumed to be UTF-8. If opening in binary mode, then every character is written
    /// and read as a four-byte big-endian value
    pub fn stream_open_file_uni(
        &mut self,
        _fileref: GlkFileRef,
        _mode: GlkFileMode,
        _rock: GlkRock,
    ) -> Option<GlkStreamID> {
        todo!();
    }
}

#[cfg(test)]
mod test {
    use crate::{windows::testwin::GlkTestWindow, GlkFileUsage};

    use super::*;

    fn get_tmpdir() -> String {
        if let Ok(tmpdir) = std::env::var("TMPDIR") {
            tmpdir.to_string()
        } else {
            "/tmp".to_string()
        }
    }

    #[test]
    fn can_read_char_buffer_from_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let test_string = "testing";
        let mut buf = Vec::new();
        for ch in test_string.chars() {
            buf.push((ch as u32 >> 24) as u8);
            buf.push(((ch as u32 >> 16) & 0xff) as u8);
            buf.push(((ch as u32 >> 8) & 0xff) as u8);
            buf.push((ch as u32 & 0xff) as u8);
        }
        let mem_stream = glk.stream_open_memory(buf, GlkFileMode::Read, 45);

        assert_eq!(glk.get_buffer_stream_uni(mem_stream, None), "testing");
    }

    #[test]
    fn can_read_char_from_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let unibuf = vec!['t', 'e', 's', 't', 'i', 'n', 'g'];

        let mut buf = Vec::new();
        for ch in unibuf {
            buf.push((ch as u32 >> 24) as u8);
            buf.push(((ch as u32 >> 16) & 0xff) as u8);
            buf.push(((ch as u32 >> 8) & 0xff) as u8);
            buf.push((ch as u32 & 0xff) as u8);
        }

        let mem_stream = glk.stream_open_memory(buf, GlkFileMode::Read, 45);
        assert_eq!(glk.get_char_stream_uni(mem_stream), Some('t'));
    }

    #[test]
    fn can_read_a_line_of_bytes_from_a_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let test_string = "testing line 1\ntesting line 2\ntesting line 3\n";

        let mut buf = Vec::new();
        for ch in test_string.chars() {
            buf.push(ch as u8);
        }
        let mem_stream = glk.stream_open_memory(buf, GlkFileMode::Read, 45);

        assert_eq!(
            glk.get_line_stream(mem_stream, None),
            "testing line 1"
                .chars()
                .map(|c| c as u8)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            glk.get_line_stream(mem_stream, None),
            "testing line 2"
                .chars()
                .map(|c| c as u8)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn can_read_byte_buffer_from_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let mem_stream = glk.stream_open_memory(
            vec![b't', b'e', b's', b't', b'i', b'n', b'g'],
            GlkFileMode::Read,
            45,
        );

        assert_eq!(
            glk.get_buffer_stream(mem_stream, None),
            "testing".chars().map(|c| c as u8).collect::<Vec<_>>()
        );
    }

    #[test]
    fn can_read_byte_from_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let mem_stream = glk.stream_open_memory(vec![b't'], GlkFileMode::Read, 45);
        assert_eq!(glk.get_char_stream(mem_stream), Some(b't'));
    }

    #[test]
    fn can_read_a_line_of_chars_from_a_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let test_string = "testing line 1\ntesting line 2\ntesting line 3\n";

        let mut buf = Vec::new();
        for ch in test_string.chars() {
            buf.push((ch as u32 >> 24) as u8);
            buf.push(((ch as u32 >> 16) & 0xff) as u8);
            buf.push(((ch as u32 >> 8) & 0xff) as u8);
            buf.push((ch as u32 & 0xff) as u8);
        }
        let mem_stream = glk.stream_open_memory(buf, GlkFileMode::Read, 45);

        assert_eq!(glk.get_line_stream_uni(mem_stream, None), "testing line 1");
        assert_eq!(glk.get_line_stream_uni(mem_stream, None), "testing line 2");
    }

    #[test]
    fn can_get_stream_position() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let mem_stream = glk.stream_open_memory(
            vec![b't', b'e', b's', b't', b'i', b'n', b'g'],
            GlkFileMode::Read,
            45,
        );

        assert_eq!(glk.stream_get_position(mem_stream).unwrap(), 0);
        glk.get_char_stream(mem_stream);
        assert_eq!(glk.stream_get_position(mem_stream).unwrap(), 1);
    }

    #[test]
    fn can_seek_within_memory_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let mem_stream = glk.stream_open_memory(
            vec![b't', b'e', b's', b't', b'i', b'n', b'g'],
            GlkFileMode::Read,
            45,
        );

        glk.stream_set_position(mem_stream, 4, GlkSeekMode::Start);
        assert_eq!(glk.get_char_stream(mem_stream), Some(b'i'));

        glk.stream_set_position(mem_stream, -4, GlkSeekMode::End);
        assert_eq!(glk.get_char_stream(mem_stream), Some(b't'));

        glk.stream_set_position(mem_stream, -2, GlkSeekMode::Current);
        assert_eq!(glk.get_char_stream(mem_stream), Some(b's'));

        assert!(glk
            .stream_set_position(mem_stream, -2, GlkSeekMode::Start)
            .is_none());
        assert!(glk
            .stream_set_position(mem_stream, 2, GlkSeekMode::End)
            .is_none());

        let close = glk.stream_close(mem_stream);
        assert!(close.is_some());

        if let Some((result, bytes)) = close {
            assert_eq!(result.read_count, 3);
            assert_eq!(result.write_count, 0);
            assert_eq!(bytes, Some(vec![b't', b'e', b's', b't', b'i', b'n', b'g']));
        }
    }

    #[test]
    fn can_open_a_file_and_write_to_it() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let fileref = glk.fileref_create_temp(GlkFileUsage::Data, 23).unwrap();
        let stream = glk
            .stream_open_file(fileref, GlkFileMode::ReadWrite, 24)
            .unwrap();
        glk.put_string_stream(stream, "This is a test of a temp file");
        glk.stream_set_position(stream, 0, GlkSeekMode::Start);
        let result = glk
            .get_line_stream(stream, None)
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        assert_eq!(result, "This is a test of a temp file".to_string());
    }

    #[test]
    fn can_write_to_a_non_temp_file() {
        let tmpfile = format!("{}/io_file.txt", get_tmpdir());
        let mut glk = Glk::<GlkTestWindow>::new();
        let fileref = glk
            .fileref_create_by_name(GlkFileUsage::Data, tmpfile, 23)
            .unwrap();
        let stream = glk
            .stream_open_file(fileref, GlkFileMode::Write, 24)
            .unwrap();
        glk.put_string_stream(stream, "This is a test of a named file");
        let response = glk.stream_close(stream);
        assert!(response.is_some());

        if let Some((result, bytes)) = response {
            assert!(bytes.is_none());
            assert_eq!(result.read_count, 0);
            assert_eq!(result.write_count, 30);
        }

        let stream = glk
            .stream_open_file(fileref, GlkFileMode::Read, 24)
            .unwrap();
        let result = glk
            .get_line_stream(stream, None)
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        assert_eq!(result, "This is a test of a named file".to_string());

        glk.fileref_delete_file(fileref);
    }

    #[test]
    fn can_append_to_a_file() {
        let tmpfile = format!("{}/append_file.txt", get_tmpdir());
        let mut glk = Glk::<GlkTestWindow>::new();
        let fileref = glk
            .fileref_create_by_name(GlkFileUsage::Data, tmpfile, 23)
            .unwrap();
        let stream = glk
            .stream_open_file(fileref, GlkFileMode::Write, 24)
            .unwrap();
        glk.put_string_stream(stream, "This is a test of an appended file\n");
        glk.stream_close(stream);

        let stream = glk
            .stream_open_file(fileref, GlkFileMode::WriteAppend, 24)
            .unwrap();
        glk.put_string_stream(stream, "This is the second line of an appended file\n");
        glk.stream_close(stream);

        let stream = glk
            .stream_open_file(fileref, GlkFileMode::Read, 24)
            .unwrap();
        let result = glk
            .get_buffer_stream(stream, None)
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        assert_eq!(
            result,
            "This is a test of an appended file\nThis is the second line of an appended file\n"
                .to_string()
        );

        glk.stream_set_position(stream, 0, GlkSeekMode::Start);
        let result = glk
            .get_buffer_stream(stream, Some(5))
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        assert_eq!(result, "This ");

        glk.fileref_delete_file(fileref);
    }

    #[test]
    fn can_read_multiple_lines_from_a_file() {
        let tmpfile = format!("{}/multi_line_file.txt", get_tmpdir());
        let mut glk = Glk::<GlkTestWindow>::new();
        let fileref = glk
            .fileref_create_by_name(GlkFileUsage::Data, tmpfile, 23)
            .unwrap();
        let stream = glk
            .stream_open_file(fileref, GlkFileMode::Write, 24)
            .unwrap();

        glk.put_string_stream(stream, "Line 1\n");
        glk.put_string_stream(stream, "Line 2\n");
        glk.put_string_stream(stream, "Line 3\n");
        glk.stream_close(stream);

        let stream = glk
            .stream_open_file(fileref, GlkFileMode::Read, 24)
            .unwrap();

        let result = glk
            .get_line_stream(stream, None)
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        assert_eq!(result, "Line 1\n");

        // should be able to read a partial line
        let result = glk
            .get_line_stream(stream, Some(3))
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        assert_eq!(result, "Lin");

        // should be able to stop at a newline even if requesting more characters
        let result = glk
            .get_line_stream(stream, Some(10))
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        assert_eq!(result, "e 2\n");

        let result = glk
            .get_line_stream(stream, None)
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        assert_eq!(result, "Line 3\n");

        glk.stream_close(stream);
    }

    #[test]
    fn can_read_and_write_utf8_characters() {
        let tmpfile = format!("{}/utf8_file.txt", get_tmpdir());
        let mut glk = Glk::<GlkTestWindow>::new();

        let fileref = glk
            .fileref_create_by_name(GlkFileUsage::Data, tmpfile, 23)
            .unwrap();
        let stream = glk
            .stream_open_file(fileref, GlkFileMode::Write, 24)
            .unwrap();

        glk.put_string_stream(stream, "some ascii text...");

        // two-byte unicode
        let sset = 'ß';
        glk.put_char_stream_uni(stream, sset);

        // three-byte unicode
        let horns = 'ࢠ';
        glk.put_char_stream_uni(stream, horns);

        let flower = '🌸';
        glk.put_char_stream_uni(stream, flower);

        glk.put_string_stream(stream, "some trailing text?");
        glk.stream_close(stream);

        let stream = glk
            .stream_open_file(fileref, GlkFileMode::Read, 25)
            .unwrap();

        for _ in 0..18 {
            let _ = glk.get_char_stream_uni(stream);
        }

        let input = glk.get_char_stream_uni(stream).unwrap();
        assert_eq!(input, sset);

        let input = glk.get_char_stream_uni(stream).unwrap();
        assert_eq!(input, horns);

        let input = glk.get_char_stream_uni(stream).unwrap();
        assert_eq!(input, flower);

        // TODO: read a string from a file
        // let input = glk.get_buffer_stream_uni(stream, None);
        // assert_eq!(input, "some trailing text?".to_string());
    }
}
