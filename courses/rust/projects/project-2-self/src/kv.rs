use std::collections::{BTreeMap, HashMap};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use serde::{Serialize, Deserialize};
use serde_json;
use std::io::prelude::*;
use std::io::{self, SeekFrom, Seek, Read, Write, BufReader, BufWriter};
use std::ffi::OsStr;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
enum Operation {
    Set{ k: String, v: String },
    Rm { k: String },
}

struct BufReaderWithPos<R: Seek + Read> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Seek + Read> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        inner.seek(SeekFrom::Start(0))?;
        let pos = 0;
        Ok(Self {
            reader: BufReader::new(inner),
            pos
        })
    }
}

impl<R: Seek + Read> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Seek + Read> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<W: Seek + Write> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Seek + Write> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        inner.seek(SeekFrom::Start(0))?;
        let pos = 0;
        Ok(Self {
            writer: BufWriter::new(inner),
            pos
        })
    }
}

impl<W: Seek + Write> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

impl<W: Seek + Write> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// The `KvStore` stores string key/value pairs.
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// ```
pub struct KvStore {
    path: PathBuf,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    writer: BufWriterWithPos<File>,
    current_gen: u64,
    index: BTreeMap<String, CommandPos>,
    uncompacted: u64,
}

struct CommandPos {
    id: u64,
    pos: u64,
    len: u64,
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{gen}.log"))
}

fn load(gen: u64, reader: &mut BufReaderWithPos<File>, index: &mut BTreeMap<String, CommandPos>) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = serde_json::Deserializer::from_reader(reader).into_iter::<Operation>();
    let mut uncompacted = 0;
    while let Some(Ok(cmd)) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd {
            Operation::Set{k, ..} => {
                if let Some(old_cmd) = index.insert(k, CommandPos{id: gen, pos: pos, len: new_pos - pos}) {
                    uncompacted += old_cmd.len;
                }
            }
            Operation::Rm{k} => {
                if let Some(old_cmd) = index.remove(&k) {
                    uncompacted += old_cmd.len;
                }
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(uncompacted)
}

fn new_log_file(path: &Path, gen: u64, readers: &mut HashMap<u64, BufReaderWithPos<File>>) -> Result<BufWriterWithPos<File>> {
    let p = log_path(&path, gen);
    let mut writer = BufWriterWithPos::new(File::options().create(true).append(true).write(true).open(p)?)?;
    readers.insert(gen, BufReaderWithPos::new(File::open(log_path(&path, gen))?)?);
    Ok(writer)
}

fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut gen_list: Vec<u64> = fs::read_dir(&path)?
        .flat_map(|res| -> Result<_> {Ok(res?.path())})
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path|
            {
                path.file_name().and_then(OsStr::to_str).map(|s| s.trim_end_matches(".log")).map(str::parse::<u64>)
            })
            .flatten()
            .collect();

    gen_list.sort_unstable();
    Ok(gen_list)
}

impl KvStore {
    /// open
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        let gen_list = sorted_gen_list(&path)?;
        let mut uncompacted = 0;
        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, current_gen, &mut readers)?;

        Ok(Self {
            path: path,
            readers: readers,
            writer: writer,
            current_gen: current_gen,
            index: index,
            uncompacted: uncompacted,
        })
    }

    /// Set
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // insert into writer
        let cmd = Operation::Set{k: key, v: value};
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Operation::Set{k, ..} = cmd {
            let len = self.writer.pos - pos;
            let command_pos = CommandPos{id: self.current_gen, pos: pos, len: len};
            if let Some(old_cmd) = self.index.insert(k, command_pos) {
                self.uncompacted += old_cmd.len;
            }
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    /// Set
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(&CommandPos{id, pos, len}) = self.index.get(&key) {
            let mut r = self.readers.get_mut(&id).expect("SDSD");
            r.seek(SeekFrom::Start(pos))?;
            let cmd_reader = r.take(len);
            if let Operation::Set{v, ..} = serde_json::from_reader(cmd_reader)? {
                return Ok(Some(v));
            } else {
                // return Err("not found".to_owned());
            }

        }
        return Ok(None);
    }

    /// Set
    pub fn remove(&mut self, key: String) -> Result<()> {
        if let Some(_) = self.index.get(&key) {
            let cmd = Operation::Rm{k: key.clone()};
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            self.index.remove(&key).expect("ADSD");
        } else {
            return Err(anyhow::anyhow!("UH"));
        }
        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = new_log_file(&self.path, self.current_gen, &mut self.readers)?;

        let mut compaction_writer = new_log_file(&self.path, compaction_gen, &mut self.readers)?;

        let mut new_pos = 0;
        for cmd_pos in &mut self.index.values_mut() {
            let mut reader = self.readers.get_mut(&cmd_pos.id).expect("sdasd");
            if cmd_pos.pos != reader.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }
            // copy it to current
            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *cmd_pos = CommandPos{id: compaction_gen, pos: new_pos, len: len};
            new_pos += len;
        }
        compaction_writer.flush()?;

        let stale_gens: Vec<_> = self
            .readers
            .keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned()
            .collect();
        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }
        self.uncompacted = 0;

        Ok(())
    }
}