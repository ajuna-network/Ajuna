// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#[cfg(any(feature = "ajuna", feature = "bajun"))]
use crate::cli::RelayChainCli;
use crate::{
	cli::{Cli, Subcommand},
	command_helper::{inherent_benchmark_data, BenchmarkExtrinsicBuilder},
};
use ajuna_service::{
	ajuna_solo_runtime::Block as SoloBlock,
	chain_spec,
	solo::{self, new_full, new_partial, ExecutorDispatch},
};

use frame_benchmarking_cli::BenchmarkCmd;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use sc_service::PartialComponents;
use std::{path::PathBuf, sync::Arc};

fn load_spec(id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match id {
		"dev" =>
			Box::new(chain_spec::solo::development_config(sc_service::ChainType::Development)?),
		"testnet" => Box::new(chain_spec::solo::testnet_config()?),
		"" | "local" =>
			Box::new(chain_spec::solo::development_config(sc_service::ChainType::Local)?),
		path => Box::new(chain_spec::solo::ChainSpec::from_json_file(PathBuf::from(path))?),
	})
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Ajuna Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/ajuna-network/Ajuna/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2022
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&ajuna_service::ajuna_solo_runtime::VERSION
	}
}

#[cfg(any(feature = "ajuna", feature = "bajun"))]
impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Parachain Collator Template".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Parachain Collator Template\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relay chain node.\n\n\
		parachain-collator <parachain-args> -- <relay-chain-args>"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/cumulus/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2022
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		runner.async_run(|$config| {
            let $components = solo::new_partial(&$config)?;
            let task_manager = $components.task_manager;
            { $( $code )* }.map(|v| (v, task_manager))
		})
	}}
}

macro_rules! construct_sync_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		runner.sync_run(|$config| {
            let $components = solo::new_partial(&$config)?;
            { $( $code )* }
		})
	}}
}

/// Parse and run command line arguments
#[allow(unused_variables)]
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.database))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.chain_spec))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::PurgeChainSolo(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, backend, .. } = new_partial(&config)?;
				let aux_revert = Box::new(|client, _, blocks| {
					sc_finality_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			// Switch on the concrete benchmark sub-command-
			match cmd {
				BenchmarkCmd::Pallet(cmd) => runner.sync_run(|config| {
					if cfg!(feature = "runtime-benchmarks") {
						cmd.run::<SoloBlock, ExecutorDispatch>(config)
					} else {
						Err("Runtime benchmarking wasn't enabled when building the node. \
                            You can enable it with `--features runtime-benchmarks`."
							.into())
					}
				}),
				BenchmarkCmd::Block(cmd) => {
					construct_sync_run!(|components, cli, cmd, config| {
						cmd.run(components.client)
					})
				},
				BenchmarkCmd::Storage(cmd) => {
					construct_sync_run!(|components, cli, cmd, config| {
						let db = components.backend.expose_db();
						let storage = components.backend.expose_storage();
						cmd.run(config, components.client, db, storage)
					})
				},
				BenchmarkCmd::Overhead(cmd) => runner.sync_run(|config| {
					let PartialComponents { client, .. } = new_partial(&config)?;
					let ext_builder = BenchmarkExtrinsicBuilder::new(client.clone());
					cmd.run(config, client, inherent_benchmark_data()?, Arc::new(ext_builder))
				}),
				BenchmarkCmd::Machine(cmd) => {
					construct_sync_run!(|components, cli, cmd, config| cmd.run(&config))
				},
			}
		},
		#[cfg(any(feature = "ajuna", feature = "bajun"))]
		Some(_) => Err("Unsupported command for solo chain".into()),
		None => {
			let runner = cli.create_runner(&cli.run_solo)?;
			runner.run_node_until_exit(|config| async move {
				new_full(config).map_err(sc_cli::Error::Service)
			})
		},
	}
}
