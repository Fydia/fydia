use futures::prelude::*;
use fydia_utils::generate_string;
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{self, BufReader, Read, Write},
};

use crate::messages::Date;

const PREFIX: &str = "./storage/";

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDescriptor {
    pub name: String,
    pub date: Date,
}

impl FileDescriptor {
    pub fn new<T: Into<String>>(name: T, date: Date) -> Self {
        Self {
            name: name.into(),
            date,
        }
    }
    pub fn new_with_now<T: Into<String>>(name: T) -> Self {
        Self::new(name, Date::now())
    }
    pub fn to_string(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(v) => Ok(v),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub struct File {
    path: String,
    description: Option<String>,
}

impl Default for File {
    fn default() -> Self {
        Self::get(generate_string(32))
    }
}

impl File {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_name(&self) -> String {
        self.path
            .strip_prefix(PREFIX)
            .unwrap_or_default()
            .to_string()
    }

    pub fn create(&self) -> Result<(), String> {
        println!("{}", self.path);
        drop(std::fs::create_dir(PREFIX));
        std::fs::File::create(&self.path)
            .map_err(|f| f.to_string())
            .map(|_| ())
    }

    pub fn create_with_description(&self, file_descriptor: FileDescriptor) -> Result<(), String> {
        println!("{}", self.path);
        drop(std::fs::create_dir(PREFIX));
        std::fs::File::create(&self.path)
            .map_err(|f| f.to_string())
            .map(|_| ())?;

        let mut file =
            std::fs::File::create(format!("{}.json", &self.path)).map_err(|f| f.to_string())?;

        file.write(file_descriptor.to_string()?.as_bytes())
            .map(|_| ())
            .map_err(|error| error.to_string())
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
            description: None,
        }
    }

    pub fn get_with_description<T: Into<String>>(path_to_file: T) -> Self {
        let path = format!("{}{}", PREFIX, &path_to_file.into());
        let description = Some(format!("{}.json", &path));

        Self { path, description }
    }

    pub fn write_value(&self, buf: &mut [u8]) -> Result<(), String> {
        let mut file = std::fs::File::open(&self.path).map_err(|f| f.to_string())?;
        file.read(buf)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    pub fn get_value(&self) -> Result<Vec<u8>, String> {
        let file = std::fs::File::open(&self.path).map_err(|f| f.to_string())?;
        let buf = BufReader::new(file);
        Ok(buf.buffer().to_vec())
    }

    pub fn async_get_value(&self) -> Result<impl Stream<Item = Result<u8, io::Error>>, String> {
        let file = std::fs::File::open(&self.path).map_err(|f| f.to_string())?;
        let buf = BufReader::new(file);
        Ok(stream::iter(buf.bytes()))
    }

    pub fn get_description(&self) -> Result<FileDescriptor, String> {
        let value = self.get_value_of_description()?;
        let string = String::from_utf8(value).map_err(|f| f.to_string())?;

        serde_json::from_str::<FileDescriptor>(&string).map_err(|f| f.to_string())
    }

    pub fn get_value_of_description(&self) -> Result<Vec<u8>, String> {
        let desc = self
            .description
            .as_ref()
            .ok_or_else(|| "No file".to_string())?;

        let file = std::fs::File::open(desc).map_err(|f| f.to_string())?;
        let buf = BufReader::new(file);
        Ok(buf.buffer().to_vec())
    }

    pub fn async_get_value_of_description(
        &self,
    ) -> Result<impl Stream<Item = Result<u8, io::Error>>, String> {
        let desc = self
            .description
            .as_ref()
            .ok_or_else(|| "No file".to_string())?;
        let file = std::fs::File::open(&desc).map_err(|f| f.to_string())?;
        let buf = BufReader::new(file);
        Ok(stream::iter(buf.bytes()))
    }
}
