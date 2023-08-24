use mktemp::Temp;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::{
    stream::{GlkStreamResult, StreamHandler},
    GlkFileMode, GlkFileUsage, GlkRock,
};

/// A reference to a file
pub type GlkFileRef = u32;

#[derive(Default, Debug)]
pub(crate) struct FileRefManager {
    fileref: HashMap<GlkFileRef, FileRef>,
    val: GlkFileRef,
}

impl FileRefManager {
    pub(crate) fn get(&self, id: GlkFileRef) -> Option<&FileRef> {
        self.fileref.get(&id)
    }

    pub(crate) fn create_temp_file(
        &mut self,
        usage: GlkFileUsage,
        rock: GlkRock,
    ) -> Option<GlkFileRef> {
        self.create_named_file(usage, Temp::new_file().unwrap().to_path_buf(), rock)
    }

    pub(crate) fn create_named_file(
        &mut self,
        usage: GlkFileUsage,
        name: PathBuf,
        rock: GlkRock,
    ) -> Option<GlkFileRef> {
        self.fileref.insert(
            self.val,
            FileRef {
                _usage: usage,
                name,
                _rock: rock,
                is_temp: true,
            },
        );

        self.val += 1;

        Some(self.val - 1)
    }
}

/// A reference to a file
#[derive(Clone, Debug)]
pub(crate) struct FileRef {
    /// The usage of the file
    _usage: GlkFileUsage,

    /// The name of the file
    name: PathBuf,

    /// The file reference rock
    _rock: GlkRock,

    /// are we creating a temporary file
    pub(crate) is_temp: bool,
}

impl FileRef {}

#[derive(Debug)]
pub(crate) struct FileStream {
    _fileref: FileRef,
    _rock: GlkRock,
    fp: Option<File>,
    result: GlkStreamResult,
}

impl FileStream {
    pub(crate) fn create_temp(fileref: &FileRef, rock: GlkRock) -> Option<Self> {
        let fp = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(fileref.name.clone())
            .ok()?;

        Some(Self {
            _fileref: fileref.clone(),
            _rock: rock,
            fp: Some(fp),
            result: GlkStreamResult::default(),
        })
    }

    pub(crate) fn open_file(fileref: &FileRef, mode: GlkFileMode, rock: GlkRock) -> Option<Self> {
        let fp = OpenOptions::new()
            .read(mode.is_read())
            .write(mode.is_write())
            .create(mode != GlkFileMode::Read)
            .truncate(mode == GlkFileMode::Write)
            .open(fileref.name.clone())
            .ok()?;

        Some(Self {
            _fileref: fileref.clone(),
            _rock: rock,
            fp: Some(fp),
            result: GlkStreamResult::default(),
        })
    }
}

impl StreamHandler for FileStream {
    fn close(&mut self) {
        let _ = self.fp.take();
    }

    fn put_char(&mut self, ch: u8) {
        if let Some(fp) = self.fp.as_mut() {
            let _ = write!(fp, "{ch}");
        }
    }

    fn put_string(&mut self, s: &str) {
        if let Some(fp) = self.fp.as_mut() {
            let _ = write!(fp, "{s}");
        }
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
        let fp = if let Some(fp) = self.fp.as_ref() {
            fp.try_clone()
        } else {
            return Vec::new();
        };
        let file_clone = if let Ok(cloned) = fp {
            cloned
        } else {
            return Vec::new();
        };

        let mut br = BufReader::new(file_clone);

        let _ = br.read_line(&mut result);

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
        if let Some(fp) = self.fp.as_mut() {
            fp.seek(seek_to).ok()?;
        }
        Some(())
    }

    fn get_data(&self) -> Vec<u8> {
        todo!()
    }

    fn is_window_stream(&self) -> bool {
        false
    }

    fn is_memory_stream(&self) -> bool {
        false
    }

    fn increment_output_count(&mut self, count: usize) {
        self.result.write_count += count as u32;
    }

    fn increment_input_count(&mut self, count: usize) {
        self.result.read_count += count as u32;
    }

    fn get_results(&self) -> crate::stream::GlkStreamResult {
        self.result.clone()
    }
}
