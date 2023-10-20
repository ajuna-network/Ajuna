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

#![allow(deprecated)]
#![allow(unreachable_code)]

use crate::para::cli::{Cli, RelayChainCli, Subcommand};
use ajuna_primitives::Block;
#[cfg(feature = "ajuna")]
use ajuna_service::{
	ajuna_runtime::{Block as AjunaBlock, RuntimeApi as AjunaRuntimeApi},
	para::AjunaRuntimeExecutor,
};
#[cfg(feature = "bajun")]
use ajuna_service::{
	bajun_runtime::{Block as BajunBlock, RuntimeApi as BajunRuntimeApi},
	para::BajunRuntimeExecutor,
};
use ajuna_service::{chain_spec, para as service};
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use log::info;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_runtime::traits::AccountIdConversion;
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
	} else {
		Err("Chain spec (solo, bajun, ajuna) must be specified".into())
	}
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		if cfg!(feature = "bajun") {
			"Bajun Node".into()
		} else {
			"Ajuna Node".into()
		}
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
		"https://github.com/ajuna-network/Ajuna".into()
	}

	fn copyright_start_year() -> i32 {
		2021
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}
}

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
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		runner.async_run(|$config| {
			#[cfg(feature = "bajun")]
            let $components = service::new_partial::<BajunRuntimeApi, BajunRuntimeExecutor, _>(
				&$config,
				service::aura_build_import_queue::<BajunRuntimeApi, BajunRuntimeExecutor>
			)?;

			#[cfg(feature = "ajuna")]
            let $components = service::new_partial::<AjunaRuntimeApi, AjunaRuntimeExecutor, _>(
				&$config,
				service::aura_build_import_queue::<AjunaRuntimeApi, AjunaRuntimeExecutor>
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
			#[cfg(feature = "bajun")]
			let $components = service::new_partial::<BajunRuntimeApi, BajunRuntimeExecutor, _>(
				&$config,
				service::aura_build_import_queue::<BajunRuntimeApi, BajunRuntimeExecutor>
			)?;

			#[cfg(feature = "ajuna")]
			let $components = service::new_partial::<AjunaRuntimeApi, AjunaRuntimeExecutor, _>(
				&$config,
				service::aura_build_import_queue::<AjunaRuntimeApi, AjunaRuntimeExecutor>
			)?;

            { $( $code )* }
		})
	}}
}

/// Parse command line arguments into service configuration.
#[allow(unused_variables)]
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
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
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.backend, None))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
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
		Some(Subcommand::ExportGenesisState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				#[cfg(feature = "ajuna")]
				{
					let partials = service::new_partial::<AjunaRuntimeApi, AjunaRuntimeExecutor, _>(
						&config,
						service::aura_build_import_queue::<AjunaRuntimeApi, AjunaRuntimeExecutor>,
					)?;
					return cmd.run::<Block>(&*config.chain_spec, &*partials.client)
				}
				#[cfg(feature = "bajun")]
				{
					let partials = service::new_partial::<BajunRuntimeApi, BajunRuntimeExecutor, _>(
						&config,
						service::aura_build_import_queue::<BajunRuntimeApi, BajunRuntimeExecutor>,
					)?;
					cmd.run::<Block>(&*config.chain_spec, &*partials.client)
				}
				#[cfg(not(feature = "bajun"))]
				panic!("No runtime feature (bajun, ajuna) is enabled")
			})
		},
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				cmd.run(&*spec)
			})
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			// Switch on the concrete benchmark sub-command-
			match cmd {
				BenchmarkCmd::Pallet(cmd) =>
					if cfg!(feature = "runtime-benchmarks") {
						match &runner.config().chain_spec {
							#[cfg(feature = "ajuna")]
							spec if spec.id().starts_with("ajuna") =>
								runner.sync_run(|config| cmd.run::<AjunaBlock, ()>(config)),
							#[cfg(feature = "bajun")]
							spec if spec.id().starts_with("bajun") =>
								runner.sync_run(|config| cmd.run::<BajunBlock, ()>(config)),
							_ => panic!("No runtime feature (bajun, ajuna) is enabled"),
						}
					} else {
						Err("Benchmarking wasn't enabled when building the node. \
					You can enable it with `--features runtime-benchmarks`."
							.into())
					},
				BenchmarkCmd::Block(cmd) =>
					construct_sync_run!(|components, cli, cmd, config| cmd.run(components.client)),
				#[cfg(not(feature = "runtime-benchmarks"))]
				BenchmarkCmd::Storage(_) =>
					return Err(sc_cli::Error::Input(
						"Compile with --features=runtime-benchmarks \
						to enable storage benchmarks."
							.into(),
					)
					.into()),
				#[cfg(feature = "runtime-benchmarks")]
				BenchmarkCmd::Storage(cmd) => construct_sync_run!(|partials, cli, cmd, config| {
					let db = partials.backend.expose_db();
					let storage = partials.backend.expose_storage();
					cmd.run(config, partials.client.clone(), db, storage)
				}),
				BenchmarkCmd::Machine(cmd) =>
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
				// NOTE: this allows the Client to leniently implement
				// new benchmark commands without requiring a companion MR.
				#[allow(unreachable_patterns)]
				_ => Err("Benchmarking sub-command unsupported".into()),
			}
		},
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			use sc_executor::{sp_wasm_interface::ExtendedHostFunctions, NativeExecutionDispatch};
			use try_runtime_cli::block_building_info::timestamp_with_aura_info;

			let runner = cli.create_runner(cmd)?;
			type HostFunctionsOf<E> = ExtendedHostFunctions<
				sp_io::SubstrateHostFunctions,
				<E as NativeExecutionDispatch>::ExtendHostFunctions,
			>;

			// grab the task manager.
			let registry = &runner.config().prometheus_config.as_ref().map(|cfg| &cfg.registry);
			let task_manager =
				sc_service::TaskManager::new(runner.config().tokio_handle.clone(), *registry)
					.map_err(|e| format!("Error: {:?}", e))?;

			#[cfg(feature = "ajuna")]
			if cfg!(feature = "ajuna") {
				return runner.async_run(|_| {
					Ok((
						cmd.run::<AjunaBlock, HostFunctionsOf<AjunaRuntimeExecutor>, _>(Some(
							timestamp_with_aura_info::<AjunaBlock>(6000),
						)),
						task_manager,
					))
				})
			}
			#[cfg(feature = "bajun")]
			{
				#[allow(clippy::needless_return)]
				return runner.async_run(|_| {
					Ok((
						cmd.run::<BajunBlock, HostFunctionsOf<BajunRuntimeExecutor>, _>(Some(
							timestamp_with_aura_info::<BajunBlock>(6000),
						)),
						task_manager,
					))
				})
			}
			#[cfg(not(feature = "bajun"))]
			panic!("No runtime feature (bajun, ajuna) is enabled")
		},
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("Try-runtime was not enabled when building the node. \
			You can enable it with `--features try-runtime`."
			.into()),
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();

			runner.run_node_until_exit(|config| async move {
				let hwbench = if !cli.no_hardware_benchmarks {
					config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(database_path);
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
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);

				let id = ParaId::from(para_id);

				let parachain_account =
					AccountIdConversion::<polkadot_primitives::AccountId>::into_account_truncating(
						&id,
					);

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", id);
				info!("Parachain Account: {}", parachain_account);
				info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

				match &config.chain_spec {
					#[cfg(feature = "ajuna")]
					spec if spec.id().starts_with("ajuna") =>
						service::start_lookahead_parachain_node::<
							AjunaRuntimeApi,
							AjunaRuntimeExecutor,
						>(config, polkadot_config, collator_options, id, hwbench)
						.await
						.map(|r| r.0)
						.map_err(Into::into),
					#[cfg(feature = "bajun")]
					spec if spec.id().starts_with("bajun") =>
						service::start_lookahead_parachain_node::<
							BajunRuntimeApi,
							BajunRuntimeExecutor,
						>(config, polkadot_config, collator_options, id, hwbench)
						.await
						.map(|r| r.0)
						.map_err(Into::into),
					_ => panic!("No runtime feature (bajun, ajuna) is enabled"),
				}
			})
		},
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_listen_port() -> u16 {
		9945
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()?
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool(is_dev)
	}

	fn trie_cache_maximum_size(&self) -> Result<Option<usize>> {
		self.base.base.trie_cache_maximum_size()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}
}
