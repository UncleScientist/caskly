use mktemp::Temp;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::{
    stream::{GlkStreamHandler, GlkStreamResult},
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
        self.create_file(usage, Temp::new_file().unwrap().to_path_buf(), rock, true)
    }

    pub(crate) fn create_named_file(
        &mut self,
        usage: GlkFileUsage,
        name: PathBuf,
        rock: GlkRock,
    ) -> Option<GlkFileRef> {
        self.create_file(usage, name, rock, false)
    }

    fn create_file(
        &mut self,
        usage: GlkFileUsage,
        name: PathBuf,
        rock: GlkRock,
        is_temp: bool,
    ) -> Option<GlkFileRef> {
        self.fileref.insert(
            self.val,
            FileRef {
                _usage: usage,
                name,
                _rock: rock,
                is_temp,
            },
        );

        self.val += 1;

        Some(self.val - 1)
    }

    pub(crate) fn delete_file_by_id(&mut self, id: GlkFileRef) {
        if let Some(file) = self.fileref.get(&id) {
            let _ = std::fs::remove_file(&file.name);
        }
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
    input_buf: Option<BufReader<File>>,
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
            input_buf: None,
        })
    }

    pub(crate) fn open_file(fileref: &FileRef, mode: GlkFileMode, rock: GlkRock) -> Option<Self> {
        let mut options = OpenOptions::new();
        let options = options
            .read(mode.is_read())
            .write(mode.is_write())
            .append(mode == GlkFileMode::WriteAppend)
            .create(mode != GlkFileMode::Read)
            .truncate(mode == GlkFileMode::Write);

        let fp = options.open(fileref.name.clone()).ok()?;

        Some(Self {
            _fileref: fileref.clone(),
            _rock: rock,
            fp: Some(fp),
            result: GlkStreamResult::default(),
            input_buf: None,
        })
    }

    fn get_bufreader(&mut self) -> &mut BufReader<File> {
        if self.input_buf.is_none() {
            self.input_buf = Some(BufReader::new(
                self.fp.as_ref().unwrap().try_clone().unwrap(),
            ));
        }

        if let Some(br) = self.input_buf.as_mut() {
            br
        } else {
            panic!("!");
        }
    }
}

impl GlkStreamHandler for FileStream {
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

    fn put_char_uni(&mut self, ch: char) {
        let mut bytes = [0u8; 4];
        let len = ch.encode_utf8(&mut bytes).len();

        if let Some(fp) = self.fp.as_mut() {
            let _ = fp.write(&bytes[0..len]);
        }
    }

    fn put_buffer_uni(&mut self, buf: &[char]) {
        for ch in buf {
            self.put_char_uni(*ch);
        }
    }

    fn get_char(&mut self) -> Option<u8> {
        let br = self.get_bufreader();
        let mut buf = [0u8];
        if br.read(&mut buf).is_ok() {
            Some(buf[0])
        } else {
            None
        }
    }

    fn get_buffer(&mut self, maxlen: Option<usize>) -> Vec<u8> {
        let Some(mut fp) = self.fp.as_ref() else {
            return Vec::new();
        };

        if let Some(maxlen) = maxlen {
            let mut buf = vec![0u8; maxlen];
            let _ = fp.read(&mut buf);
            buf
        } else {
            let mut buf: Vec<u8> = Vec::new();
            let _ = fp.read_to_end(&mut buf);
            buf
        }
    }

    fn get_line(&mut self, maxlen: Option<usize>) -> Vec<u8> {
        let mut result = String::from("");

        let br = self.get_bufreader();

        let _ = if let Some(maxlen) = maxlen {
            let mut buf = vec![0u8; maxlen];

            if br.read_exact(&mut buf).is_err() {
                return Vec::new();
            };

            if let Some(pos) = buf.iter().position(|x| *x == b'\n') {
                let seek_to = (maxlen - pos) as i64 - 1;
                let _ = br.seek_relative(-seek_to);
                return buf.into_iter().take(pos + 1).collect::<Vec<u8>>();
            }

            result = buf.into_iter().map(|x| x as char).collect::<String>();
            Ok(result.len())
        } else {
            br.read_line(&mut result)
        };

        result.chars().map(|x| x as u8).collect()
    }

    fn get_char_uni(&mut self) -> Option<char> {
        todo!()
    }

    fn get_buffer_uni(&mut self, _maxlen: Option<usize>) -> String {
        todo!()
    }

    fn get_line_uni(&mut self, _maxlen: Option<usize>) -> String {
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
