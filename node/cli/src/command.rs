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

#[cfg(feature = "ajuna")]
use ajuna_service::{
	ajuna,
	ajuna::AjunaRuntimeExecutor,
	ajuna_runtime::{Block as ParaAjunaBlock, RuntimeApi as AjunaRuntimeApi},
};
#[cfg(feature = "bajun")]
use ajuna_service::{
	bajun,
	bajun::BajunRuntimeExecutor,
	bajun_runtime::{Block as ParaBajunBlock, RuntimeApi as BajunRuntimeApi},
};
#[cfg(any(feature = "bajun", feature = "ajuna"))]
use {
	crate::cli::RelayChainCli, codec::Encode, cumulus_client_cli::generate_genesis_block,
	cumulus_primitives_core::ParaId, log::info, sp_core::hexdisplay::HexDisplay,
	sp_runtime::traits::AccountIdConversion, sp_runtime::traits::Block as BlockT,
};

#[cfg(feature = "solo")]
use {
	crate::{
		benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder},
		cli::{Cli, Subcommand},
	},
	ajuna_service::{
		ajuna_solo_runtime::{Block as SoloBlock, ExistentialDeposit},
		chain_spec,
		solo::{self, new_full, new_partial, ExecutorDispatch},
	},
};

use frame_benchmarking_cli::*;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use sc_service::PartialComponents;
use sp_keyring::Sr25519Keyring;
use std::path::PathBuf;

fn load_spec(id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	if cfg!(feature = "bajun") {
		#[cfg(feature = "bajun")]
		{
			Ok(match id {
				"dev" => Box::new(chain_spec::bajun::development_config()),
				"" | "local" => Box::new(chain_spec::bajun::local_testnet_config()),
				path =>
					Box::new(chain_spec::bajun::ChainSpec::from_json_file(PathBuf::from(path))?),
			})
		}
		#[cfg(not(feature = "bajun"))]
		return Err("Chain spec for Bajun doesn't exist".into())
	} else if cfg!(feature = "ajuna") {
		#[cfg(feature = "ajuna")]
		{
			Ok(match id {
				"dev" => Box::new(chain_spec::ajuna::development_config()),
				"" | "local" => Box::new(chain_spec::ajuna::local_testnet_config()),
				path =>
					Box::new(chain_spec::ajuna::ChainSpec::from_json_file(PathBuf::from(path))?),
			})
		}
		#[cfg(not(feature = "ajuna"))]
		return Err("Chain spec for Ajuna doesn't exist".into())
	} else if cfg!(feature = "solo") {
		#[cfg(feature = "solo")]
		{
			Ok(match id {
				"dev" => Box::new(chain_spec::solo::development_config(
					sc_service::ChainType::Development,
				)?),
				"testnet" => Box::new(chain_spec::solo::testnet_config()?),
				"" | "local" =>
					Box::new(chain_spec::solo::development_config(sc_service::ChainType::Local)?),
				path => Box::new(chain_spec::solo::ChainSpec::from_json_file(PathBuf::from(path))?),
			})
		}
		#[cfg(not(feature = "solo"))]
		return Err("Solo chain spec doesn't exist".into())
	} else {
		Err("Chain spec (solo, bajun, ajuna) must be specified".into())
	}
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		#[cfg(feature = "bajun")]
		if cfg!(feature = "bajun") {
			return "Bajun Node".into()
		}
		#[cfg(feature = "ajuna")]
		if cfg!(feature = "ajuna") {
			return "Ajuna Node".into()
		}
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
		2021
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		#[cfg(feature = "bajun")]
		if cfg!(feature = "bajun") {
			return &ajuna_service::bajun_runtime::VERSION
		}
		#[cfg(feature = "ajuna")]
		if cfg!(feature = "ajuna") {
			return &ajuna_service::ajuna_runtime::VERSION
		}
		&ajuna_service::ajuna_solo_runtime::VERSION
	}
}

#[cfg(any(feature = "bajun", feature = "ajuna"))]
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
		2020
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
            #[cfg(feature = "solo")]
            let $components = solo::new_partial(&$config)?;

			#[cfg(feature = "bajun")]
            let $components = bajun::new_partial::<BajunRuntimeApi, BajunRuntimeExecutor, _>(
                &$config,
                bajun::parachain_build_import_queue,
            )?;

			#[cfg(feature = "ajuna")]
            let $components = ajuna::new_partial::<AjunaRuntimeApi, AjunaRuntimeExecutor, _>(
                &$config,
                ajuna::parachain_build_import_queue,
            )?;

            let task_manager = $components.task_manager;
            { $( $code )* }.map(|v| (v, task_manager))
		})
	}}
}

macro_rules! construct_sync_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		runner.sync_run(|$config| {
            #[cfg(feature = "solo")]
            let $components = solo::new_partial(&$config)?;

			#[cfg(feature = "bajun")]
            let $components = bajun::new_partial::<BajunRuntimeApi, BajunRuntimeExecutor, _>(
                &$config,
                bajun::parachain_build_import_queue,
            )?;

			#[cfg(feature = "ajuna")]
            let $components = ajuna::new_partial::<AjunaRuntimeApi, AjunaRuntimeExecutor, _>(
                &$config,
                ajuna::parachain_build_import_queue,
            )?;

            { $( $code )* }
		})
	}}
}

/// Parse command line arguments into service configuration.
#[allow(unused_variables)]
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		#[cfg(any(feature = "bajun", feature = "ajuna"))]
		Some(Subcommand::ExportGenesisState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
			let state_version = Cli::native_runtime_version(&spec).state_version();
			#[cfg(feature = "bajun")]
			if cfg!(feature = "bajun") {
				return runner.sync_run(|_config| cmd.run::<ParaBajunBlock>(&*spec, state_version))
			}
			#[cfg(feature = "ajuna")]
			if cfg!(feature = "ajuna") {
				return runner.sync_run(|_config| cmd.run::<ParaAjunaBlock>(&*spec, state_version))
			}
			Err("This subcommand should only work under bajun / ajuna features.".into())
		},
		#[cfg(any(feature = "bajun", feature = "ajuna"))]
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				cmd.run(&*spec)
			})
		},
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
		#[cfg(feature = "solo")]
		Some(Subcommand::PurgeChainSolo(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},
		#[cfg(any(feature = "bajun", feature = "ajuna"))]
		Some(Subcommand::PurgeChainPara(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);

				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		},
		Some(Subcommand::Revert(cmd)) => {
			#[cfg(any(feature = "bajun", feature = "ajuna"))]
			if cfg!(feature = "bajun") || cfg!(feature = "ajuna") {
				return construct_async_run!(|components, cli, cmd, config| {
					Ok(cmd.run(components.client, components.backend, None))
				})
			}
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
				BenchmarkCmd::Overhead(cmd) => {
					if cfg!(feature = "bajun") {
						return Err("Unsupported benchmarking command".into())
					}
					runner.sync_run(|config| {
						let PartialComponents { client, .. } = new_partial(&config)?;
						let ext_builder = RemarkBuilder::new(client.clone());
						cmd.run(config, client, inherent_benchmark_data()?, &ext_builder)
					})
				},
				BenchmarkCmd::Extrinsic(cmd) => {
					runner.sync_run(|config| {
						let PartialComponents { client, .. } = new_partial(&config)?;
						// Register the *Remark* and *TKA* builders.
						let ext_factory = ExtrinsicFactory(vec![
							Box::new(RemarkBuilder::new(client.clone())),
							Box::new(TransferKeepAliveBuilder::new(
								client.clone(),
								Sr25519Keyring::Alice.to_account_id(),
								ExistentialDeposit::get(),
							)),
						]);

						cmd.run(client, inherent_benchmark_data()?, &ext_factory)
					})
				},
				BenchmarkCmd::Machine(cmd) => {
					construct_sync_run!(|_components, cli, cmd, config| cmd
						.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()))
				},
			}
		},
		None => {
			#[cfg(feature = "bajun")]
			if cfg!(feature = "bajun") {
				let runner = cli.create_runner(&cli.run_para.normalize())?;
				let collator_options = cli.run_para.collator_options();

				return runner.run_node_until_exit(|config| async move {
					let hwbench = if !cli.no_hardware_benchmarks {
						config.database.path().map(|database_path| {
							let _ = std::fs::create_dir_all(&database_path);
							sc_sysinfo::gather_hwbench(Some(database_path))
						})
					} else {
						None
					};

					let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
						.map(|e| e.para_id)
						.ok_or("Could not find parachain ID in chain-spec.")?;

					let polkadot_cli = RelayChainCli::new(
						&config,
						[RelayChainCli::executable_name()]
							.iter()
							.chain(cli.relay_chain_args.iter()),
					);

					let id = ParaId::from(para_id);

					let parachain_account =
						AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account_truncating(
							&id,
						);

					let state_version =
						RelayChainCli::native_runtime_version(&config.chain_spec).state_version();
					let block: ParaBajunBlock =
						generate_genesis_block(&*config.chain_spec, state_version)
							.map_err(|e| format!("{:?}", e))?;
					let genesis_state =
						format!("0x{:?}", HexDisplay::from(&block.header().encode()));

					let tokio_handle = config.tokio_handle.clone();
					let polkadot_config = SubstrateCli::create_configuration(
						&polkadot_cli,
						&polkadot_cli,
						tokio_handle,
					)
					.map_err(|err| format!("Relay chain argument error: {}", err))?;

					info!("Parachain id: {:?}", id);
					info!("Parachain Account: {}", parachain_account);
					info!("Parachain genesis state: {}", genesis_state);
					info!(
						"Is collating: {}",
						if config.role.is_authority() { "yes" } else { "no" }
					);
					bajun::start_parachain_node(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				})
			}
			#[cfg(feature = "ajuna")]
			if cfg!(feature = "ajuna") {
				let runner = cli.create_runner(&cli.run_para.normalize())?;
				let collator_options = cli.run_para.collator_options();

				return runner.run_node_until_exit(|config| async move {
					let hwbench = if !cli.no_hardware_benchmarks {
						config.database.path().map(|database_path| {
							let _ = std::fs::create_dir_all(&database_path);
							sc_sysinfo::gather_hwbench(Some(database_path))
						})
					} else {
						None
					};

					let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
						.map(|e| e.para_id)
						.ok_or("Could not find parachain ID in chain-spec.")?;

					let polkadot_cli = RelayChainCli::new(
						&config,
						[RelayChainCli::executable_name()]
							.iter()
							.chain(cli.relay_chain_args.iter()),
					);

					let id = ParaId::from(para_id);

					let parachain_account =
						AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account_truncating(
							&id,
						);

					let state_version =
						RelayChainCli::native_runtime_version(&config.chain_spec).state_version();
					let block: ParaAjunaBlock =
						generate_genesis_block(&*config.chain_spec, state_version)
							.map_err(|e| format!("{:?}", e))?;
					let genesis_state =
						format!("0x{:?}", HexDisplay::from(&block.header().encode()));

					let tokio_handle = config.tokio_handle.clone();
					let polkadot_config = SubstrateCli::create_configuration(
						&polkadot_cli,
						&polkadot_cli,
						tokio_handle,
					)
					.map_err(|err| format!("Relay chain argument error: {}", err))?;

					info!("Parachain id: {:?}", id);
					info!("Parachain Account: {}", parachain_account);
					info!("Parachain genesis state: {}", genesis_state);
					info!(
						"Is collating: {}",
						if config.role.is_authority() { "yes" } else { "no" }
					);
					ajuna::start_parachain_node(
						config,
						polkadot_config,
						collator_options,
						id,
						hwbench,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				})
			}
			let runner = cli.create_runner(&cli.run_solo)?;
			runner.run_node_until_exit(|config| async move {
				new_full(config).map_err(sc_cli::Error::Service)
			})
		},
	}
}
