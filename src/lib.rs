use std::convert::TryInto;
use std::io::{self, Seek, SeekFrom, Read};

use binread::BinRead;
use modular_bitfield::prelude::*;

#[derive(BinRead, Debug)]
#[br(magic = b"NSO0")]
pub struct NsoFile {
    pub version: u32,
    pub reserved: u32,
    pub flags: Flags,
    pub text_segment_header: SegmentHeader,
    pub module_name_offset: u32,
    pub rodata_segment_header: SegmentHeader,
    pub module_name_size: u32,
    pub data_segment_header: SegmentHeader,
    pub bss_size: u32,
    pub module_id: ModuleId,
    pub text_file_size: u32,
    pub rodata_file_size: u32,
    pub data_file_size: u32,
    pub reserved2: [u32; 7],
    pub embedded_section_header: SectionHeader,
    pub dyn_str_section_header: SectionHeader,
    pub dyn_sym_section_header: SectionHeader,
    pub text_hash: Sha256,
    pub rodata_hash: Sha256,
    pub data_hash: Sha256,
}

impl NsoFile {
    pub fn get_raw_text_reader<'a, R: Read + Seek>(&self, reader: &'a mut R) -> io::Result<impl Read + 'a> {
        reader.seek(SeekFrom::Start(self.text_segment_header.file_offset as u64))?;

        Ok(reader.take(self.text_file_size as u64))
    }

    pub fn get_text<R: Read + Seek>(&self, reader: &mut R) -> io::Result<Vec<u8>> {
        if self.flags.text_compressed() {
            let mut reader = self.get_raw_text_reader(reader)?;
            let mut compressed_data = Vec::with_capacity(self.text_file_size as usize);
            reader.read_to_end(&mut compressed_data)?;

            lz4::block::decompress(
                &compressed_data[..],
                Some(self.text_segment_header.size.try_into().unwrap())
            )
        } else {
            let mut reader = self.get_raw_text_reader(reader)?;
            let mut data = Vec::with_capacity(self.text_file_size as usize);
            reader.read_to_end(&mut data)?;

            Ok(data)
        }
    }
    
    pub fn get_raw_data_reader<'a, R: Read + Seek>(&self, reader: &'a mut R) -> io::Result<impl Read + 'a> {
        reader.seek(SeekFrom::Start(self.data_segment_header.file_offset as u64))?;

        Ok(reader.take(self.data_file_size as u64))
    }

    pub fn get_data<R: Read + Seek>(&self, reader: &mut R) -> io::Result<Vec<u8>> {
        if self.flags.data_compressed() {
            let mut reader = self.get_raw_data_reader(reader)?;
            let mut compressed_data = Vec::with_capacity(self.data_file_size as usize);
            reader.read_to_end(&mut compressed_data)?;

            lz4::block::decompress(
                &compressed_data[..],
                Some(self.data_segment_header.size.try_into().unwrap())
            )
        } else {
            let mut reader = self.get_raw_data_reader(reader)?;
            let mut data = Vec::with_capacity(self.data_file_size as usize);
            reader.read_to_end(&mut data)?;

            Ok(data)
        }
    }
    
    pub fn get_raw_rodata_reader<'a, R: Read + Seek>(&self, reader: &'a mut R) -> io::Result<impl Read + 'a> {
        reader.seek(SeekFrom::Start(self.rodata_segment_header.file_offset as u64))?;

        Ok(reader.take(self.rodata_file_size as u64))
    }

    pub fn get_rodata<R: Read + Seek>(&self, reader: &mut R) -> io::Result<Vec<u8>> {
        if self.flags.rodata_compressed() {
            let mut reader = self.get_raw_rodata_reader(reader)?;
            let mut compressed_rodata = Vec::with_capacity(self.data_file_size as usize);
            reader.read_to_end(&mut compressed_rodata)?;

            lz4::block::decompress(
                &compressed_rodata[..],
                Some(self.rodata_segment_header.size.try_into().unwrap())
            )
        } else {
            let mut reader = self.get_raw_rodata_reader(reader)?;
            let mut rodata = Vec::with_capacity(self.rodata_file_size as usize);
            reader.read_to_end(&mut rodata)?;

            Ok(rodata)
        }
    }
}

type ModuleId = [u8; 32];
type Sha256 = [u8; 32];

#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct Flags {
    pub text_compressed: bool,
    pub rodata_compressed: bool,
    pub data_compressed: bool,
    pub text_hash: bool,
    pub rodata_hash: bool,
    pub data_hash: bool,
    pub reserved: B26,
}

#[derive(BinRead, Debug)]
pub struct SegmentHeader {
    pub file_offset: u32,
    pub memory_offset: u32,
    pub size: u32,
}

#[derive(BinRead, Debug)]
pub struct SectionHeader {
    pub file_offset: u32,
    pub size: u32,
}

#[cfg(test)]
mod tests {
    use super::NsoFile;
    use binread::BinReaderExt;

    #[test]
    fn parse_test() {
        let test_path = "/home/jam/re/ult/901/main";
        
        let mut file = std::io::Cursor::new(std::fs::read(test_path).unwrap());

        let nso: NsoFile = file.read_le().unwrap();

        println!("{:#X?}", nso);
        println!("{:#X}", nso.get_text(&mut file).unwrap().len());
    }
}
