use std::{fs::File, io::Read};
use std::os::unix::prelude::FileExt;

const MIN_SEQ_SIZE: usize = 5;
const MAX_SEQ_SIZE: usize = 10;

#[derive(Clone)]
#[derive(Debug)]
pub enum AddressType {
    Original,
    Added,
}

pub enum Mode {
    Normal,
    Insert,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Seq {
    pub address: usize,
    pub address_type: AddressType,
    pub length: usize,
    pub total_length: usize,
}

pub struct Buffer {
    pub orig_data: Vec<u8>,
    pub file_name: String,
    pub file: File,
    pub cursor: usize,
    pub add_data: Vec<u8>,
    pub seq: Vec<Seq>,
    pub size: usize,
}

impl Buffer {
    pub fn open(file_name: &str) -> Result<Buffer, std::io::Error> {
        let mut file = File::open(file_name)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let mut buff = Buffer {
            orig_data: data.clone(),
            file_name: file_name.to_string(),
            file,
            cursor: 0,
            add_data: Vec::new(),
            seq: Vec::new(),
            size: data.len(),
        };
        buff.seq.push(Seq {
            address: 0,
            total_length: data.len(),
            address_type: AddressType::Original,
            length: data.len(),
        });
        Ok(buff)
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        let mut at = 0;

        // calculate the new size
        let mut len = bytes.len() * 2;
        if len < MIN_SEQ_SIZE {
            len = MIN_SEQ_SIZE;
        }
        if len > MAX_SEQ_SIZE {
            len = MAX_SEQ_SIZE;
        }
        let mut count = 0;
        // insert data if space is available
        for seq in &mut self.seq {
            count += seq.length;
            println!("space check seq: {:?}", seq);
            if self.cursor <= count && bytes.len() <= seq.total_length - seq.length {
                let move_from = self.cursor - (count - seq.length);
                let move_to = move_from + bytes.len();
                // shift all the data to the right
                let source = self.add_data[move_from..move_from + bytes.len()].to_vec();
                self.add_data[move_to..move_to + bytes.len()].copy_from_slice(&source);
                // insert the new data
                self.add_data[move_from..move_from + bytes.len()].copy_from_slice(bytes);
                seq.length += bytes.len();
                return Ok(());
            }
            if self.cursor <= count {
                break;
            }
        }

        // allocate space at the end of add_data
        println!("curr len: {}", self.add_data.len());
        println!("alocating extra len: {}", len);
        let old_size = self.add_data.len();
        at = old_size;
        println!("before add_data: -{:?}", self.add_data);
        self.add_data.resize(len + old_size, 0);
        println!("len after allocation: {}", self.add_data.len());
        self.add_data[at..at + bytes.len()].copy_from_slice(bytes); 
        println!("add_data: -{:?}", self.add_data);
        let new_seq = Seq {
            address: at,
            address_type: AddressType::Added,
            length: bytes.len(),
            total_length: len,
        };

        if self.cursor == 0 { // if at the start
            self.seq.insert(0, new_seq);
        } else if self.cursor == self.size { // if at the end
            self.seq.push(new_seq);
        } else { // if in the middle
            let mut i = 0;
            let mut count = 0;
            let mut offset = 0;
            // find the seq that the self.cursor is in
            for seq in &self.seq {
                count += seq.length;
                if self.cursor <= count {
                    offset = self.cursor - (count - seq.length);
                    break;
                }
                i += 1;
            }
            println!("offset: {}, i: {}", offset, i);
            // split the seq
            let mut third_seq = self.seq[i].clone();
            third_seq.address += offset ;
            third_seq.length -= offset ;
            // update the first seq
            self.seq[i].length = offset ;
            println!("first_seq: {:?}", self.seq[i]);
            println!("second_seq: {:?}", new_seq);
            println!("third_seq: {:?}", third_seq);
            if third_seq.length != 0 {
                self.seq.insert(i + 1, third_seq);
            }
            // insert the new seq
            self.seq.insert(i + 1, new_seq);
            if offset == 0 {
                self.seq.remove(i);
            }
        }
        // update the size
        self.size += bytes.len();
        self.cursor += bytes.len();
        Ok(())
    }

    pub fn get_file_create(file_name: &str) -> Result<File, std::io::Error> {
        let result = File::open(file_name);
        if result.is_ok() {
            return result;
        }
        File::create(file_name)
    }
   
    pub fn save_buffer_as(&mut self, file_name: &str) -> Result<(), std::io::Error> {
        let mut bytes_written = 0;
        let file = if file_name != self.file_name 
                    { Buffer::get_file_create(file_name).unwrap() } 
                    else { self.file.try_clone().unwrap() };
        for seq in &self.seq {
            let data = match seq.address_type {
                AddressType::Original => &self.orig_data[seq.address..(seq.address + seq.length)],
                AddressType::Added => &self.add_data[seq.address..(seq.address + seq.length)],
            };
            file.write_at(data, bytes_written)?;
            bytes_written += seq.length as u64;
        }
        Ok(())
    }

    pub fn get_window(&self, mut start: usize, end: usize) -> Vec<u8> {
        let mut window = Vec::new();
        let mut address = 0;
        println!("added_data: {:?}", self.add_data);
        for seq in &self.seq {
            println!("seq: {:?}", seq);
            let mut bytes_read = 0;
            println!("window before = {}", std::str::from_utf8(&window).unwrap());
            if start >= address && start < address + seq.length { 
                let start_from = seq.address + start - address;
                let end_from = (seq.address + end - address).min(seq.address + seq.length);
                println!("start_from: {}, end_from: {}", start_from, end_from);
                bytes_read += end_from - start_from;
                match seq.address_type {
                    AddressType::Original => {
                        window.extend_from_slice(&self.orig_data[start_from..end_from]);
                    }
                    AddressType::Added => {
                        window.extend_from_slice(&self.add_data[start_from..end_from]);
                    }
                }
                println!("window after = {}", std::str::from_utf8(&window).unwrap());
                start += bytes_read;
            }
            if start >= end {
                return window;
            }
            address += seq.length;
        }
        window
    }
    pub fn move_cursor_to(&mut self, to: usize) {
        self.cursor = to;
    }

    fn get_seq(&self, at: usize) -> (usize, usize) {
        let mut count = 0;
        let mut i = 0;
        for seq in &self.seq {
            count += seq.length;
            if at <= count {
                return (i, at - (count - seq.length));
            }
            i += 1;
        }
        (i, 0)
    }

    pub fn delete_n_bytes_from(&mut self, mut n: usize, mut from: usize) -> Result<(), std::io::Error> {
        println!("deleting bytes..");
        while n > 0 {
            let (seq_i, _offset) = self.get_seq(from);
            let mut seq = self.seq[seq_i].clone();
            println!("Seq: {:?}", seq);
            println!("offset: {}", _offset);
            println!("n: {}", n);
            println!("seq_i: {}", seq_i);
            if _offset  <= n  {
                if _offset != seq.length {
                    let start_from = seq.address + _offset ;
                    let end_from = seq.address + seq.length;
                    let start_to = seq.address;
                    let end_to = seq.address + end_from - start_from;
                    match seq.address_type {
                        AddressType::Original => {
                            let data_to_move = self.orig_data[start_from..end_from].to_vec();
                            self.orig_data[start_to..end_to].copy_from_slice(&data_to_move);
                        }
                        AddressType::Added => {
                            let data_to_move = self.add_data[start_from..end_from].to_vec();
                            self.add_data[start_to..end_to].copy_from_slice(&data_to_move);
                        }
                    }
                    seq.length -= _offset;
                    println!("Updated Seq: {:?}", seq);
                    self.seq[seq_i] = seq;
                } else {
                    self.seq.remove(seq_i);
                }
            } else {
                #[allow(unused_assignments)]
                if _offset != seq.length {
                    let start_from = seq.address + _offset ;
                    let end_from = seq.address +  seq.length;
                    let start_to = start_from - n ;
                    let end_to = end_from - n ;
                    match seq.address_type {
                        AddressType::Original => {
                            let data_to_move = self.orig_data[start_from..end_from].to_vec();
                            self.orig_data[start_to..end_to].copy_from_slice(&data_to_move);
                        }
                        AddressType::Added => {
                            let data_to_move = self.add_data[start_from..end_from].to_vec();
                            self.add_data[start_to..end_to].copy_from_slice(&data_to_move);
                        }
                    }
                    seq.length -= n ;
                    println!("Updated Seq: {:?}", seq);
                    self.seq[seq_i] = seq;
                } else {
                    self.seq.remove(seq_i);
                }
            }
            println!("Added Data: {:?}", self.add_data);
            let t = n.min(_offset);
            n -= t;
            from -= t;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open() {
        let result = Buffer::open("test_assets/dummy_file.txt");
        assert!(result.is_ok());
        println!("bytes read {}", result.unwrap().size);
        let result = Buffer::open("not_a_file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_window() {
        let result = Buffer::open("test_assets/dummy_file.txt");
        assert!(result.is_ok());
        let s = String::from("Hello World!");
        let buffer = result.unwrap();
        let output = buffer.get_window(0, s.len());
        let output = std::str::from_utf8(&output);
        let output = output.unwrap();
        println!("Expected:\n{}\nActual:\n{}", s, output);
        assert_eq!(s, output);
    }
    
    #[test]
    fn test_write_n_delete_n_save() {
        let result = Buffer::open("test_assets/dummy_file.txt");
        assert!(result.is_ok());
        let mut buffer = result.unwrap();
        let s_new = "Rhythm";
        let s = "Hello World!";
        buffer.move_cursor_to(0);
        buffer.write_bytes(s_new.as_bytes()).unwrap();
        let expected = format!("{}{}", s_new, s);
        let output = buffer.get_window(0, expected.len());
        let output = std::str::from_utf8(&output);
        let output = output.unwrap();
        println!("Expected:\n{}\nActual:\n{}", expected, output);
        assert_eq!(expected, output);
        buffer.move_cursor_to(s_new.len() - 1);
        buffer.write_bytes(s_new.as_bytes()).unwrap();
        let expected = format!("{}{}", "RhythRhythmm", s);
        let output = buffer.get_window(0, expected.len());
        let output = std::str::from_utf8(&output);
        let output = output.unwrap();
        println!("Expected:\n{}\nActual:\n{}", expected, output);
        assert_eq!(expected, output);
        buffer.move_cursor_to(s_new.len() - 1);
        buffer.write_bytes(s_new.as_bytes()).unwrap();
        let expected = "RhythRhythmRhythmm";
        let output = buffer.get_window(0, expected.len());
        let output = std::str::from_utf8(&output);
        let output = output.unwrap();
        println!("Expected:\n{}\nActual:\n{}", expected, output);
        assert_eq!(expected, output);
        println!("Added data: {:?}", buffer.add_data);
        println!("Sequence Vec: {:?}", buffer.seq);

        let _ = buffer.delete_n_bytes_from(16, 16);
        let output = buffer.get_window(0, 14);
        let output = std::str::from_utf8(&output);
        let output = output.unwrap();
        let expected = "mmHello World!";
        println!("Added data: {:?}", buffer.add_data);
        println!("Sequence Vec: {:?}", buffer.seq);
        println!("Expected:\n{}\nActual:\n{}", expected, output);
        assert_eq!(expected, output);

        //test saving it
        let _ = buffer.save_buffer_as("test_assets/dummy_file_new.txt");
    }
}

