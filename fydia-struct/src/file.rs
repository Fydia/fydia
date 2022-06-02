//! This module is related to file

use futures::prelude::*;
use fydia_utils::generate_string;
use fydia_utils::{
    serde::{Deserialize, Serialize},
    serde_json,
};
use std::{
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Read, Write},
};

use crate::messages::Date;

const PREFIX: &str = "./storage/";

/// `FileDescriptor` is used to describe a file.
///
/// When a User send a file message in a Channel,
/// the file will be stock with a random name and `FileDescriptor` is here to
/// back name of the file and date too.
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct FileDescriptor {
    pub name: String,
    pub date: Date,
}

impl FileDescriptor {
    /// Take name as a generic value that impl `Into<String>`
    /// and date as `Date` to return `FileDescriptor`
    ///
    ///# Examples
    ///```
    /// use fydia_struct::{file::FileDescriptor, messages::Date};
    ///
    /// let file_descriptor = FileDescriptor::new("name", Date::now());
    ///```
    pub fn new<T: Into<String>>(name: T, date: Date) -> Self {
        Self {
            name: name.into(),
            date,
        }
    }

    /// Take name as a generic value that impl `Into<String>`
    /// to return `FileDescriptor`
    ///
    ///# Examples
    ///```
    /// use fydia_struct::file::FileDescriptor;
    ///
    /// let file_descriptor = FileDescriptor::new_with_now("name");
    ///```
    pub fn new_with_now<T: Into<String>>(name: T) -> Self {
        Self::new(name, Date::now())
    }

    /// Serialize `FileDescriptor` in Json and return a `Result<String, String>`
    ///
    /// # Errors
    /// Return an error if :
    /// * `FileDescriptor` cannot be serialized
    ///
    ///# Examples
    ///```
    /// use fydia_struct::{file::FileDescriptor, messages::Date};
    /// use chrono::MIN_DATETIME;
    ///
    /// let file_descriptor = FileDescriptor::new("name", Date::new(MIN_DATETIME));
    /// let json_file_descriptor = file_descriptor.to_string().unwrap();
    ///
    /// assert_eq!(json_file_descriptor, r#"{"name":"name","date":-8334632851200}"#)
    ///```
    pub fn to_string(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(v) => Ok(v),
            Err(error) => Err(error.to_string()),
        }
    }
}

/// `File` is used to stock file.
///  
/// `path` is the path of file.
///
/// `description` is the path of a `DescriptorFile`.
#[allow(missing_docs)]
#[derive(Debug)]
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
    /// Return `File` with a random name
    pub fn new() -> Self {
        Self::default()
    }

    /// Return name of `File`
    pub fn get_name(&self) -> String {
        self.path
            .strip_prefix(PREFIX)
            .unwrap_or_default()
            .to_string()
    }

    /// Create the `File` and the directory if it does not exist.
    ///
    /// # Errors
    /// Return an error if :
    /// * directory cannot be create
    /// * file cannot be created
    pub fn create(&self) -> Result<(), String> {
        drop(std::fs::create_dir(PREFIX).map_err(|error| error.to_string()));
        std::fs::File::create(&self.path)
            .map_err(|f| f.to_string())
            .map(|_| ())
    }

    /// Create the `File` with the `FileDescriptor`
    /// and the directory if it does not exist.
    ///
    /// # Errors
    /// Return an error if :
    /// * directory cannot be create
    /// * file cannot be created
    /// * `FileDescriptor` cannot be created
    /// * buffer cannot be written
    pub fn create_with_description(&self, file_descriptor: &FileDescriptor) -> Result<(), String> {
        self.create()?;

        let mut file =
            std::fs::File::create(format!("{}.json", &self.path)).map_err(|f| f.to_string())?;

        file.write(file_descriptor.to_string()?.as_bytes())
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    /// Create the `File` and write buffer of `[u8]`
    ///
    /// # Errors
    /// Return an error if :
    /// * file cannot be created
    /// * buffer cannot be written
    pub fn create_and_write(&self, bytes: &[u8]) -> Result<(), String> {
        self.create()?;
        self.write(bytes)
    }

    /// Write a buffer of `[u8]` to `File`
    ///
    /// # Errors
    /// Return an error if :
    /// * file cannot be created
    /// * file cannot be written
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(&self.path)
            .map_err(|f| f.to_string())?;

        file.write_all(bytes).map_err(|f| f.to_string())
    }

    /// Take the name of file to return `File`
    pub fn get<T: Into<String>>(name_file: T) -> Self {
        Self {
            path: format!("{PREFIX}{}", &name_file.into()),
            description: None,
        }
    }
    /// Take the name of file to return `File` with `Some(String)` in `File.description`
    pub fn get_with_description<T: Into<String>>(path_to_file: T) -> Self {
        let path = format!("{PREFIX}{}", &path_to_file.into());
        let description = Some(format!("{path}.json",));

        Self { path, description }
    }

    /// Takes a buffer to write the value of file
    ///
    /// # Errors
    /// Return an error if :
    /// * file cannot be read
    pub fn read_file(&self, buf: &mut [u8]) -> Result<(), String> {
        let mut file = std::fs::File::open(&self.path).map_err(|f| f.to_string())?;
        file.read(buf)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    /// Reads the file and returns its value as `Vec<u8>`
    ///
    /// # Errors
    /// Return an error if :
    /// * file cannot be read
    pub fn get_value(&self) -> Result<Vec<u8>, String> {
        let file = std::fs::File::open(&self.path).map_err(|f| f.to_string())?;
        let mut buf = BufReader::new(file);
        Ok(buf.fill_buf().map_err(|f| f.to_string())?.to_vec())
    }

    /// Reads the file and returns its value as `Stream<u8>`
    ///
    /// # Errors
    /// Return an error if :
    /// * file cannot be read
    pub fn async_get_value(&self) -> Result<impl Stream<Item = Result<u8, io::Error>>, String> {
        let file = std::fs::File::open(&self.path).map_err(|f| f.to_string())?;
        let buf = BufReader::new(file);
        Ok(stream::iter(buf.bytes()))
    }

    /// Get the `FileDescription` of the `File`
    ///
    /// # Errors
    /// Return an error if :
    /// * `FileDescription` isn't exist or doesn't exist
    pub fn get_description(&self) -> Result<FileDescriptor, String> {
        let value = self.get_value_of_description()?;
        let string = String::from_utf8(value).map_err(|f| f.to_string())?;

        serde_json::from_str::<FileDescriptor>(&string).map_err(|f| f.to_string())
    }

    /// Return value of `FileDescription` as `Vec<u8>`
    ///
    /// # Errors
    /// Return an error if :
    /// * `FileDescription` isn't exist or doesn't exist
    pub fn get_value_of_description(&self) -> Result<Vec<u8>, String> {
        let desc = self
            .description
            .as_ref()
            .ok_or_else(|| "No file".to_string())?;

        let file = std::fs::File::open(desc).map_err(|f| f.to_string())?;
        let mut buf = BufReader::new(file);
        Ok(buf.fill_buf().map_err(|f| f.to_string())?.to_vec())
    }

    /// Return value of `FileDescription` as `Stream<u8>`
    ///
    /// # Errors
    /// Return an error if:
    /// * `FileDescription` cannot be read or doesn't exist
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
