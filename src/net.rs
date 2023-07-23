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
use crate::config::Conf;

use std::io::{ErrorKind, Result as IOResult};
use std::net::Ipv4Addr;

use tokio::net::TcpStream;
use tokio::time;

pub async fn open_socket(port: u16, config: &Conf) -> IOResult<TcpStream> {
    loop {
        match TcpStream::connect((Ipv4Addr::LOCALHOST, port)).await {
            Ok(client) => break Ok(client),
            // TODO
            // Err(e) if e.raw_os_error() == Some(winerror::ERROR_PIPE_BUSY as i32) => (),
            Err(e) if config.poll && e.kind() == ErrorKind::NotFound => (),
            Err(e) => break Err(e),
        };

        time::sleep(config.poll_interval).await;
    }
}

// pub async fn do_copy()
