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

use tokio::io::{AsyncRead, AsyncWrite};

type DataIn = dyn AsyncRead + Unpin + Send + 'static;
type DataOut = dyn AsyncWrite + Unpin + Send + 'static;

pub mod config;
pub mod gpg;
pub mod net;
pub mod pipe;

/*
pub async fn run<In, Out>(
    mut data_in: In,
    mut data_out: Out,
    config: Conf,
) -> Result<(), Box<dyn Error>>
where
    In: AsyncRead + Unpin + Send + 'static,
    Out: AsyncWrite + Unpin + Send + 'static,
{

type DataIn = AsyncRead + Unpin + Send + 'static;
type DataOut = AsyncWrite + Unpin + Send + 'static;

pub async fn run(
    mut data_in: DataIn,
    mut data_out: DataOut,
    config: Conf,
) -> Result<(), Box<dyn Error>> {
    */
