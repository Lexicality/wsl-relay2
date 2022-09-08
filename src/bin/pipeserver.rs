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

use clap::Parser;
use simple_logger::SimpleLogger;
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use windows_named_pipe::PipeListener;

#[derive(Parser)]
#[clap(long_about = None)]
pub struct Conf {
    /// The name of the pipe to open
    #[clap(value_parser, default_value = "test")]
    pipe_name: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();

    let pipe_name = Conf::parse().pipe_name;
    if pipe_name.contains('\\') {
        panic!("Pipe names cannot contain backslashes!")
    }
    let pipe_path = format!(r#"\\.\pipe\{}"#, &pipe_name);

    log::info!("Starting the listen server at {}", pipe_path);

    let mut stream = PipeListener::bind(Path::new(&pipe_path))?;

    for stream in stream.into_iter() {
        let stream = stream?;
        log::info!("Got a connection!");
        let mut buf = String::new();

        let mut reader = BufReader::new(stream);
        loop {
            let res = reader.read_line(&mut buf);
            if let Result::Err(e) = res {
                log::error!("can't read {}", e);
                break;
            }
            buf = buf.trim().to_string();
            log::info!("Got message: {}", buf);
            if buf == "death" {
                log::info!("terminating early");
                let writer = reader.get_mut();
                writer.write_all("bye\n".as_bytes())?;
                writer.flush()?;
                break;
            }
            let writer = reader.get_mut();
            let answer = format!("> {}\n", &buf);
            let res = writer.write_all(answer.as_bytes());
            if let Result::Err(e) = res {
                log::error!("can't write {}", e);
                break;
            }
            buf.clear();
        }
    }
    Ok(())
}
