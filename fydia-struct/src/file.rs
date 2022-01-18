use futures::prelude::*;
use fydia_utils::generate_string;
use std::{
    fs::OpenOptions,
    io::{BufReader, Read, Write},
};

const PREFIX: &str = "./storage/";

pub struct File {
    path: String,
}

impl File {
    pub fn new() -> Self {
        Self::get(generate_string(32))
    }

    pub fn get_name(&self) -> String {
        self.path
            .strip_prefix(PREFIX)
            .unwrap_or_default()
            .to_string()
    }

    pub fn create(&self) -> Result<(), String> {
        println!("{}", self.path);
        std::fs::File::create(&self.path)
            .map_err(|f| f.to_string())
            .map(|_| ())
    }
    pub fn create_and_write(&self, bytes: Vec<u8>) -> Result<(), String> {
        self.create()?;
        self.write(bytes)?;
        Ok(())
    }

    pub fn write(&self, bytes: Vec<u8>) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(&self.path)
            .map_err(|f| f.to_string())?;

        file.write(&bytes).map_err(|f| f.to_string())?;
        Ok(())
    }

    pub fn get<T: Into<String>>(path_to_file: T) -> Self {
        Self {
            path: format!("{}{}", PREFIX, &path_to_file.into()),
        }
    }

    pub fn get_value(&self) -> Result<Vec<u8>, String> {
        let file = std::fs::File::open(&self.path).map_err(|f| f.to_string())?;
        let buf = BufReader::new(file);
        Ok(buf.buffer().to_vec())
    }

    pub fn async_get_value(
        &self,
    ) -> Result<impl Stream<Item = Result<u8, std::io::Error>>, String> {
        let file = std::fs::File::open(&self.path).map_err(|f| f.to_string())?;
        let buf = BufReader::new(file);
        Ok(stream::iter(buf.bytes()))
    }
}
