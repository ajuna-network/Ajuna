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

use std::{marker::PhantomData, sync::Arc, time::Duration};

use ajuna_primitives::{AccountId, Balance, Block, Hash, Index as Nonce};
use cumulus_client_cli::CollatorOptions;
use cumulus_client_collator::service::CollatorService;
use cumulus_client_consensus_aura::collators::lookahead::{self as aura, Params as AuraParams};
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_consensus_relay_chain::Verifier as RelayChainVerifier;
use cumulus_client_service::{
	build_network, build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks,
	BuildNetworkParams, CollatorSybilResistance, DARecoveryProfile, StartRelayChainTasksParams,
};
use cumulus_primitives_core::{
	relay_chain::{CollatorPair, ValidationCode},
	ParaId,
};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};
use parity_scale_codec::Codec;
use sc_client_api::Backend;
use sc_consensus::{
	import_queue::{BasicQueue, Verifier as VerifierT},
	BlockImportParams, ImportQueue,
};
use sc_executor::{
	HeapAllocStrategy, NativeElseWasmExecutor, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY,
};
use sc_network::{config::FullNetworkConfiguration, NetworkBlock};
use sc_network_sync::SyncingService;
use sc_service::{Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::{ApiExt, ConstructRuntimeApi, HeaderT};
use sp_consensus_aura::AuraApi;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::BlakeTwo256;
use substrate_prometheus_endpoint::Registry;

/// Bajun executor type.
pub struct BajunRuntimeExecutor;
#[cfg(feature = "bajun")]
impl sc_executor::NativeExecutionDispatch for BajunRuntimeExecutor {
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		bajun_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		bajun_runtime::native_version()
	}
}

/// Ajuna executor type.
pub struct AjunaRuntimeExecutor;
#[cfg(feature = "ajuna")]
impl sc_executor::NativeExecutionDispatch for AjunaRuntimeExecutor {
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		ajuna_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		ajuna_runtime::native_version()
	}
}

type ParachainExecutor<Executor> = NativeElseWasmExecutor<Executor>;

type ParachainClient<RuntimeApi, Executor> =
	TFullClient<Block, RuntimeApi, ParachainExecutor<Executor>>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport<RuntimeApi, Executor> =
	TParachainBlockImport<Block, Arc<ParachainClient<RuntimeApi, Executor>>, ParachainBackend>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, Executor, BIQ>(
	config: &Configuration,
	build_import_queue: BIQ,
) -> Result<
	PartialComponents<
		ParachainClient<RuntimeApi, Executor>,
		ParachainBackend,
		(),
		sc_consensus::DefaultImportQueue<Block>,
		sc_transaction_pool::FullPool<Block, ParachainClient<RuntimeApi, Executor>>,
		(
			ParachainBlockImport<RuntimeApi, Executor>,
			Option<Telemetry>,
			Option<TelemetryWorkerHandle>,
		),
	>,
	sc_service::Error,
>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, ParachainClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
	sc_client_api::StateBackendFor<ParachainBackend, Block>: sp_api::StateBackend<BlakeTwo256>,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
	BIQ: FnOnce(
		Arc<ParachainClient<RuntimeApi, Executor>>,
		ParachainBlockImport<RuntimeApi, Executor>,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let heap_pages = config
		.default_heap_pages
		.map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static { extra_pages: h as _ });

	let wasm = WasmExecutor::builder()
		.with_execution_method(config.wasm_method)
		.with_onchain_heap_alloc_strategy(heap_pages)
		.with_offchain_heap_alloc_strategy(heap_pages)
		.with_max_runtime_instances(config.max_runtime_instances)
		.with_runtime_cache_size(config.runtime_cache_size)
		.build();

	let executor = ParachainExecutor::<Executor>::new_with_wasm_executor(wasm);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let block_import =
		ParachainBlockImport::<RuntimeApi, Executor>::new(client.clone(), backend.clone());

	let import_queue = build_import_queue(
		client.clone(),
		block_import.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	)?;

	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: (),
		other: (block_import, telemetry, telemetry_worker_handle),
	})
}

enum BuildOnAccess<R> {
	Uninitialized(Option<Box<dyn FnOnce() -> R + Send + Sync>>),
	Initialized(R),
}

impl<R> BuildOnAccess<R> {
	fn get_mut(&mut self) -> &mut R {
		loop {
			match self {
				Self::Uninitialized(f) => {
					*self = Self::Initialized((f.take().unwrap())());
				},
				Self::Initialized(ref mut r) => return r,
			}
		}
	}
}

struct Verifier<Client, AuraId> {
	client: Arc<Client>,
	aura_verifier: BuildOnAccess<Box<dyn VerifierT<Block>>>,
	relay_chain_verifier: Box<dyn VerifierT<Block>>,
	_phantom: PhantomData<AuraId>,
}

#[async_trait::async_trait]
impl<Client, AuraId> VerifierT<Block> for Verifier<Client, AuraId>
where
	Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: AuraApi<Block, AuraId>,
	AuraId: Send + Sync + Codec,
{
	async fn verify(
		&mut self,
		block_import: BlockImportParams<Block>,
	) -> Result<BlockImportParams<Block>, String> {
		if self
			.client
			.runtime_api()
			.has_api::<dyn AuraApi<Block, AuraId>>(*block_import.header.parent_hash())
			.unwrap_or(false)
		{
			self.aura_verifier.get_mut().verify(block_import).await
		} else {
			self.relay_chain_verifier.verify(block_import).await
		}
	}
}

/// Build the import queue for Aura-based runtimes.
pub fn aura_build_import_queue<RuntimeApi, Executor>(
	client: Arc<ParachainClient<RuntimeApi, Executor>>,
	block_import: ParachainBlockImport<RuntimeApi, Executor>,
	config: &Configuration,
	telemetry_handle: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, ParachainClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
{
	let client2 = client.clone();

	let aura_verifier = move || {
		let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();

		Box::new(cumulus_client_consensus_aura::build_verifier::<
			sp_consensus_aura::sr25519::AuthorityPair,
			_,
			_,
			_,
		>(cumulus_client_consensus_aura::BuildVerifierParams {
			client: client2.clone(),
			create_inherent_data_providers: move |_, _| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot =
					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);

				Ok((slot, timestamp))
			},
			telemetry: telemetry_handle,
		})) as Box<_>
	};

	let relay_chain_verifier =
		Box::new(RelayChainVerifier::new(client.clone(), |_, _| async { Ok(()) })) as Box<_>;

	let verifier = Verifier {
		client,
		relay_chain_verifier,
		aura_verifier: BuildOnAccess::Uninitialized(Some(Box::new(aura_verifier))),
		_phantom: PhantomData,
	};

	let registry = config.prometheus_registry();
	let spawner = task_manager.spawn_essential_handle();

	Ok(BasicQueue::new(verifier, Box::new(block_import), None, &spawner, registry))
}

/// Start an aura powered parachain node which uses the lookahead collator to support async backing.
/// This node is basic in the sense that its runtime api doesn't include common contents such as
/// transaction payment. Used for aura glutton.
pub async fn start_lookahead_parachain_node<RuntimeApi, Executor>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient<RuntimeApi, Executor>>)>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, ParachainClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
{
	start_parachain_lookahead_node_impl::<RuntimeApi, Executor, _, _>(
		parachain_config,
		polkadot_config,
		collator_options,
		CollatorSybilResistance::Resistant, // Aura
		para_id,
		aura_build_import_queue::<RuntimeApi, Executor>,
		|client,
		 block_import,
		 prometheus_registry,
		 telemetry,
		 task_manager,
		 relay_chain_interface,
		 transaction_pool,
		 sync_oracle,
		 keystore,
		 relay_chain_slot_duration,
		 para_id,
		 collator_key,
		 overseer_handle,
		 announce_block,
		 backend| {
			let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

			let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool,
				prometheus_registry,
				telemetry,
			);
			let proposer = Proposer::new(proposer_factory);

			let collator_service = CollatorService::new(
				client.clone(),
				Arc::new(task_manager.spawn_handle()),
				announce_block,
				client.clone(),
			);

			let params = AuraParams {
				create_inherent_data_providers: move |_, ()| async move { Ok(()) },
				block_import,
				para_client: client.clone(),
				para_backend: backend,
				relay_client: relay_chain_interface,
				code_hash_provider: move |block_hash| {
					client.code_at(block_hash).ok().map(ValidationCode).map(|c| c.hash())
				},
				sync_oracle,
				keystore,
				collator_key,
				para_id,
				overseer_handle,
				slot_duration,
				relay_chain_slot_duration,
				proposer,
				collator_service,
				authoring_duration: Duration::from_millis(1500),
			};

			let fut = aura::run::<
				Block,
				sp_consensus_aura::sr25519::AuthorityPair,
				_,
				_,
				_,
				_,
				_,
				_,
				_,
				_,
				_,
			>(params);
			task_manager.spawn_essential_handle().spawn("aura", None, fut);

			Ok(())
		},
		hwbench,
	)
	.await
}

/// Start a parachain node, with lookahead implementation.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_parachain_lookahead_node_impl<RuntimeApi, Executor, BIQ, SC>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	sybil_resistance_level: CollatorSybilResistance,
	para_id: ParaId,
	build_import_queue: BIQ,
	start_consensus: SC,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient<RuntimeApi, Executor>>)>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, ParachainClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
	BIQ: FnOnce(
		Arc<ParachainClient<RuntimeApi, Executor>>,
		ParachainBlockImport<RuntimeApi, Executor>,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>,
	SC: FnOnce(
		Arc<ParachainClient<RuntimeApi, Executor>>,
		ParachainBlockImport<RuntimeApi, Executor>,
		Option<&Registry>,
		Option<TelemetryHandle>,
		&TaskManager,
		Arc<dyn RelayChainInterface>,
		Arc<sc_transaction_pool::FullPool<Block, ParachainClient<RuntimeApi, Executor>>>,
		Arc<SyncingService<Block>>,
		KeystorePtr,
		Duration,
		ParaId,
		CollatorPair,
		OverseerHandle,
		Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
		Arc<ParachainBackend>,
	) -> Result<(), sc_service::Error>,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
{
	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial::<RuntimeApi, Executor, BIQ>(&parachain_config, build_import_queue)?;
	let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

	let client = params.client.clone();
	let backend = params.backend.clone();

	let mut task_manager = params.task_manager;
	let (relay_chain_interface, collator_key) = build_relay_chain_interface(
		polkadot_config,
		&parachain_config,
		telemetry_worker_handle,
		&mut task_manager,
		collator_options.clone(),
		hwbench.clone(),
	)
	.await
	.map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue_service = params.import_queue.service();
	let net_config = FullNetworkConfiguration::new(&parachain_config.network);

	let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
		build_network(BuildNetworkParams {
			parachain_config: &parachain_config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			para_id,
			spawn_handle: task_manager.spawn_handle(),
			relay_chain_interface: relay_chain_interface.clone(),
			import_queue: params.import_queue,
			sybil_resistance_level,
		})
		.await?;

	if parachain_config.offchain_worker.enabled {
		use futures::FutureExt;

		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: Some(params.keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: network.clone(),
				is_validator: parachain_config.role.is_authority(),
				enable_http_requests: false,
				custom_extensions: move |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let rpc_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = ajuna_rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				deny_unsafe,
			};

			ajuna_rpc::create_full(deps).map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.keystore(),
		backend: backend.clone(),
		network,
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let announce_block = {
		let sync_service = sync_service.clone();
		Arc::new(move |hash, data| sync_service.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);

	let overseer_handle = relay_chain_interface
		.overseer_handle()
		.map_err(|e| sc_service::Error::Application(Box::new(e)))?;

	start_relay_chain_tasks(StartRelayChainTasksParams {
		client: client.clone(),
		announce_block: announce_block.clone(),
		para_id,
		relay_chain_interface: relay_chain_interface.clone(),
		task_manager: &mut task_manager,
		da_recovery_profile: if validator {
			DARecoveryProfile::Collator
		} else {
			DARecoveryProfile::FullNode
		},
		import_queue: import_queue_service,
		relay_chain_slot_duration,
		recovery_handle: Box::new(overseer_handle.clone()),
		sync_service: sync_service.clone(),
	})?;

	if validator {
		start_consensus(
			client.clone(),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			sync_service,
			params.keystore_container.keystore(),
			relay_chain_slot_duration,
			para_id,
			collator_key.expect("Command line arguments do not allow this. qed"),
			overseer_handle,
			announce_block,
			backend.clone(),
		)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}
