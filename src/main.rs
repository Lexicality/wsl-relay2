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
use wsl_relay::config::{Command, Conf};

use std::error::Error;
use std::process;

use clap::Parser;
use simple_logger::SimpleLogger;
use tokio::io;

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();
    log::info!("Hello, world!");
    let aa = Conf::parse();
    match aa.command {
        Command::Pipe { .. } => wsl_relay::pipe::run(io::stdin(), io::stdout(), aa),
        Command::GPG { .. } => panic!("TODO"),
    }
    .await?;

    log::info!("done");

    // beacuse tokio::io::stdin can block a clean shutdown if it's still open
    // when we get to this point, I'm choosing to always forcibly terminate.
    // All our various handles should be cleaned up by now.
    process::exit(0);
}
