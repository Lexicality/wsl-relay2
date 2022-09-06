// Copyright 2022 Lexi Robinson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::config::{Command, Conf};
use std::error::Error;
use std::fmt;
use std::io::{
    //ErrorKind,
    Read,
    Result as IOResult,
    Write,
};
use std::path::Path;
use std::thread;
use windows_named_pipe::PipeStream;

#[derive(Debug, Clone)]
pub struct InvalidPipeNameErr;

impl fmt::Display for InvalidPipeNameErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pipe names cannot contain backslashes")
    }
}
impl Error for InvalidPipeNameErr {}

fn get_pipe_name(mut name: &str) -> Result<String, InvalidPipeNameErr> {
    if name.starts_with("//./pipe/") {
        name = &name[9..]; //format!(r#"\\.\pipe\{}"#, name_input[10:])
    }

    if !name.contains('\\') {
        Ok(format!(r"\\.\pipe\{}", name))
    } else if !name.starts_with(r"\\.\pipe\") {
        Err(InvalidPipeNameErr {})
    } else {
        Ok(name.to_string())
    }
}

fn get_stream(name: String, config: &Conf) -> IOResult<PipeStream> {
    let path = Path::new(&name);
    loop {
        let res = PipeStream::connect(path);
        if !config.poll {
            return res;
        }
        let err = match res {
            Ok(_) => return res,
            Err(e) => e,
        };
        let ek = err.kind();
        log::error!("I don't know what's going on here tbh {} {}", err, ek);
        panic!("I'm lost and confused")
    }
}
use std::io;

pub fn do_copy(config: Conf) -> Result<(), Box<dyn Error>> {
    let (name, _close_on_eof) = match config.command {
        Command::Pipe {
            ref name,
            close_on_eof,
        } => (name, close_on_eof),
        _ => panic!("Unexpected config.command value!"),
    };
    let name = get_pipe_name(&name)?;
    let mut stream = get_stream(name, &config)?;
    let stream2 = &mut stream as *mut PipeStream;
    let th = thread::spawn(move || {
        let mut stdin = io::stdin();
        // let mut stream3 = stream2;
        let res = io::copy(&mut stdin, &mut unsafe { *stream2 });
    });
    stream.write(&[5])?;
    th.join().unwrap();
    Ok(())
}
