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
use std::io::{ErrorKind, Result as IOResult};

use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::time;
use winapi::shared::winerror;

enum Copy {
    In,
    Out,
}

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

async fn open_pipe(name: String, config: &Conf) -> IOResult<NamedPipeClient> {
    loop {
        match ClientOptions::new().open(&name) {
            Ok(client) => break Ok(client),
            Err(e) if e.raw_os_error() == Some(winerror::ERROR_PIPE_BUSY as i32) => (),
            Err(e) if config.poll && e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(e),
        }

        time::sleep(config.poll_interval).await;
    }
}

fn is_broken_pipe(e: &std::io::Error) -> bool {
    return e.kind() == ErrorKind::BrokenPipe
        || e.raw_os_error() == Some(winerror::ERROR_PIPE_NOT_CONNECTED as i32);
}

pub async fn do_copy<In, Out>(
    mut data_in: In,
    mut data_out: Out,
    config: Conf,
) -> Result<(), Box<dyn Error>>
where
    In: AsyncRead + Unpin + Send + 'static,
    Out: AsyncWrite + Unpin + Send + 'static,
{
    let (name, close_pipe_on_stdin_eof) = match config.command {
        Command::Pipe {
            ref name,
            close_on_eof,
        } => (name, close_on_eof),
        _ => panic!("Unexpected config.command value!"),
    };
    let name = get_pipe_name(&name)?;
    let (mut pipe_out, mut pipe_in) = io::split(open_pipe(name, &config).await?);

    let copy_in = tokio::spawn(async move {
        let res = io::copy(&mut data_in, &mut pipe_in).await?;

        // "A zero-byte write on a message pipe indicates that no more data is
        // coming."
        // I can't seem to find an explanation as to why a null write operation
        // means this, but everyone seems convinced that it does so here we go
        if close_pipe_on_stdin_eof {
            pipe_in.write(&[]).await?;
            pipe_in.flush().await?;
        }

        Ok(res)
    });
    let copy_out = tokio::spawn(async move { io::copy(&mut pipe_out, &mut data_out).await });

    tokio::pin!(copy_in);
    tokio::pin!(copy_out);

    let (copy_result, direction) = tokio::select! {
        res = (&mut copy_in) => (res?, Copy::In),
        res = (&mut copy_out) => (res?, Copy::Out),
    };

    match direction {
        Copy::In => {
            log::debug!("Input copy finished first");
            if let Err(e) = copy_result {
                if is_broken_pipe(&e) {
                    // TODO: should this actually be a fatal error?
                    log::warn!("Tried to write to the pipe but it's gone");
                    // If we can't write to a pipe we definitely can't read from
                    // it either, so we can immediately end the session
                    copy_out.abort();
                    _ = copy_out.await;
                    return Ok(());
                } else {
                    return Err(Box::new(e));
                }
            }

            if !config.exit_on_stdin_eof {
                let copy_result = copy_out.await?;
                if let Err(e) = copy_result {
                    if is_broken_pipe(&e) {
                        log::info!("Pipe broke successfully");
                    } else {
                        return Err(Box::new(e));
                    }
                }
            }
        }
        Copy::Out => {
            log::debug!("Output copy finished first");
            if let Err(e) = copy_result {
                if is_broken_pipe(&e) {
                    log::info!("Output finished: Pipe closed");
                    // If we can't read from a pipe we definitely can't write to
                    // it either, so we can immediately end the session
                    copy_in.abort();
                    _ = copy_in.await;
                    return Ok(());
                } else {
                    return Err(Box::new(e));
                }
            }
            // If we got to this point the pipe has cleanly closed itself and as
            // far as I can tell from the documentation once the server has
            // closed its end of the pipe the entire pipe is gone - there's no
            // way to turn a duplex pipe into a half duplex one
            //
            // However the original npiperelay program will keep trying to write
            // to it anyway, which seems weird but I assume there's method to
            // the madness.
            //
            // TODO: Test with what happens if you connect to a read-only or write-only pipe
            log::info!("Output end of pipe is EOF");
            if !config.exit_on_pipe_eof {
                log::info!("Waiting for input");
                let copy_result = copy_in.await?;
                if let Err(e) = copy_result {
                    // TODO: should this actually be a fatal error?
                    if is_broken_pipe(&e) {
                        log::info!("Input got a broken pipe which I kinda expected");
                    } else {
                        return Err(Box::new(e));
                    }
                }
            }
        }
    }
    Ok(())
}
