use mktemp::Temp;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::{
    stream::{GlkStreamResult, StreamHandler},
    GlkFileUsage, GlkRock,
};

/// A reference to a file
#[derive(Clone, Debug)]
pub struct GlkFileRef {
    /// The usage of the file
    _usage: GlkFileUsage,

    /// The name of the file
    name: PathBuf,

    /// The file reference rock
    _rock: GlkRock,
}

impl GlkFileRef {
    pub(crate) fn create_temp_file(usage: GlkFileUsage, rock: GlkRock) -> Option<Self> {
        Some(Self {
            _usage: usage,
            name: Temp::new_file().unwrap().to_path_buf(),
            _rock: rock,
        })
    }
}

#[derive(Debug)]
pub(crate) struct FileStream {
    _fileref: GlkFileRef,
    _rock: GlkRock,
    fp: File,
    result: GlkStreamResult,
}

impl FileStream {
    pub(crate) fn new(fileref: &GlkFileRef, rock: GlkRock) -> Option<Self> {
        let fp = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(fileref.name.clone())
            .ok()?;

        Some(Self {
            _fileref: fileref.clone(),
            _rock: rock,
            fp,
            result: GlkStreamResult::default(),
        })
    }
}

impl StreamHandler for FileStream {
    fn put_char(&mut self, ch: u8) {
        let _ = write!(self.fp, "{ch}");
    }

    fn put_string(&mut self, s: &str) {
        let _ = write!(self.fp, "{s}");
    }

    fn put_buffer(&mut self, _buf: &[u8]) {
        todo!()
    }

    fn put_char_uni(&mut self, _ch: char) {
        todo!()
    }

    fn put_buffer_uni(&mut self, _buf: &[char]) {
        todo!()
    }

    fn get_char(&self) -> Option<u8> {
        todo!()
    }

    fn get_buffer(&self, _maxlen: Option<usize>) -> Vec<u8> {
        todo!()
    }

    fn get_line(&self, _maxlen: Option<usize>) -> Vec<u8> {
        let mut result = String::from("");
        let file_clone = if let Ok(cloned) = self.fp.try_clone() {
            cloned
        } else {
            println!("could not clone file");
            return Vec::new();
        };

        let mut br = BufReader::new(file_clone);

        let read_result = br.read_line(&mut result);
        println!("read_result = {read_result:?}, result = {result}");

        result.chars().map(|x| x as u8).collect()
    }

    fn get_char_uni(&self) -> Option<char> {
        todo!()
    }

    fn get_buffer_uni(&self, _maxlen: Option<usize>) -> String {
        todo!()
    }

    fn get_line_uni(&self, _maxlen: Option<usize>) -> String {
        todo!()
    }

    fn get_position(&self) -> u32 {
        todo!()
    }

    fn set_position(&mut self, pos: i32, seekmode: crate::GlkSeekMode) -> Option<()> {
        let seek_to = match seekmode {
            crate::GlkSeekMode::Start if pos >= 0 => SeekFrom::Start(pos as u64),
            crate::GlkSeekMode::Current => SeekFrom::Current(pos as i64),
            crate::GlkSeekMode::End if pos <= 0 => SeekFrom::End(pos as i64),
            _ => return None,
        };
        self.fp.seek(seek_to).ok()?;
        Some(())
    }

    fn get_data(&self) -> Vec<u8> {
        todo!()
    }

    fn is_window_stream(&self) -> bool {
        todo!()
    }

    fn is_memory_stream(&self) -> bool {
        todo!()
    }

    fn increment_output_count(&mut self, count: usize) {
        self.result.write_count += count as u32;
    }

    fn increment_input_count(&mut self, count: usize) {
        self.result.read_count += count as u32;
    }

    fn get_results(&self) -> crate::stream::GlkStreamResult {
        todo!()
    }
}
