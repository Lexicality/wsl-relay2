// Copyright 2022 Lex Robinson
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
use clap::Subcommand;
use std::time::Duration;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Conf {
    #[clap(short, long, action)]
    verbose: bool,
    /// poll until the the specified thing exists
    #[clap(short, long, action)]
    poll: bool,
    /// How long to wait between polling
    #[clap(long, parse(try_from_str = parse_duration), default_value = "200ms")]
    poll_interval: Duration,
    /// close the write channel after stdin closes
    #[clap(short = 's', long = "close-pipe", action)]
    close_pipe_on_eof: bool,
    /// terminate when pipe closes, regardless of stdin state
    #[clap(long = "pipe-closes", action)]
    exit_on_pipe_eof: bool,
    /// terminate on stdin closes, regardless of pipe state
    #[clap(long = "input-closes", action)]
    exit_on_stdin_eof: bool,
    /// What to do
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Connects to a named pipe
    Pipe {
        /// The full name of the pipe (eg \\.\pipe\foo)
        #[clap(value_parser)]
        name: String,
    },
    /// Connects to a GPG agent socket
    GPG {
        /// The full name of the pipe (eg \\.\pipe\foo)
        #[clap(value_parser, default_value = "S.gpg-agent")]
        file: String,
    },
}

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=6053aacfa34b2cd57f36398a36839eb6
fn parse_duration(arg: &str) -> Result<std::time::Duration, humantime::DurationError> {
    arg.parse::<humantime::Duration>().map(Into::into)
}
