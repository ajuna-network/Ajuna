use crate::{self as pallet_wildcard, *};
use frame_support::{
	parameter_types,
	traits::{
		tokens::nonfungibles_v2::{Create, Mutate},
		AsEnsureOriginWithArg, ConstU16, ConstU64, Hooks,
	},
};

use frame_system::{EnsureRoot, EnsureSigned};
use pallet_nfts::PalletFeatures;
use sp_core::Pair;
use sp_runtime::{
	testing::{TestSignature, H256},
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlockNumber = u64;
pub type MockBalance = u128;
pub type MockNonce = u64;
pub type MockAssetId = u32;
pub type MockMomentResolution = u64;
pub type CurrencyOf<T> = <T as Config>::Currency;
pub type NftHelperOf<T> = <T as Config>::NonFungibles;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;

pub const CHAIN_ID: u16 = 1;
pub const FOREIGN_CHAIN_ID: u16 = 2;

pub const RESERVED_COLLECTION_0: MockCollectionId = 0;
pub const RESERVED_COLLECTION_1: MockCollectionId = 1;
pub const RESERVED_COLLECTION_2: MockCollectionId = 2;

pub const NATIVE_TOKEN_ID: MockAssetId = 0;

pub const MILLISECS_PER_BLOCK: u64 = 12000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system,
		Balances: pallet_balances,
		Nft: pallet_nfts,
		Assets: pallet_assets,
		Timestamp: pallet_timestamp,
		Wildcard: pallet_wildcard,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<MockBalance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type Nonce = MockNonce;
	type Block = MockBlock;
}

parameter_types! {
	pub const MockExistentialDeposit: MockBalance = 3;
}

impl pallet_balances::Config for Test {
	type Balance = MockBalance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = MockExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
}

pub type MockCollectionId = u32;
pub type MockItemId = u32;

parameter_types! {
	pub const CollectionDeposit: MockBalance = 333;
	pub const ItemDeposit: MockBalance = 33;
	pub const MetadataDepositBase: MockBalance = 0;
	pub const AttributeDepositBase: MockBalance = 0;
	pub const DepositPerByte: MockBalance = 0;
	pub const StringLimit: u32 = 128;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 64;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 10;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
	pub ConfigFeatures: PalletFeatures = PalletFeatures::all_enabled();
}

#[cfg(feature = "runtime-benchmarks")]
pub struct Helper;
#[cfg(feature = "runtime-benchmarks")]
impl<CollectionId: From<u32>, ItemId: From<u32>> pallet_nfts::BenchmarkHelper<CollectionId, ItemId>
	for Helper
{
	fn collection(i: u16) -> CollectionId {
		CollectionId::from(i as u32)
	}
	fn item(i: u16) -> ItemId {
		ItemId::from(i as u32)
	}
}

impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = MockCollectionId;
	type ItemId = MockItemId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<MockAccountId>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<MockAccountId>>;
	type Locker = ();
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = StringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type ApprovalsLimit = ApprovalsLimit;
	type ItemAttributesApprovalsLimit = ItemAttributesApprovalsLimit;
	type MaxTips = MaxTips;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = ConstU32<2>;
	type Features = ConfigFeatures;
	type OffchainSignature = MockSignature;
	type OffchainPublic = MockAccountPublic;
	pallet_nfts::runtime_benchmarks_enabled! {
		type Helper = Helper;
	}
	type WeightInfo = ();
}

parameter_types! {
	pub const AssetDeposit: MockBalance = MockBalance::MAX;
	pub const AssetAccountDeposit: MockBalance = 1_000;
	pub const ApprovalDeposit: MockBalance = 1_000;
	pub const MetadataDepositPerByte: MockBalance = 0;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct AssetHelper;
#[cfg(feature = "runtime-benchmarks")]
impl<AssetId: From<u32>> pallet_assets::BenchmarkHelper<AssetId> for AssetHelper {
	fn create_asset_id_parameter(id: u32) -> AssetId {
		id.into()
	}
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = MockBalance;
	type RemoveItemsLimit = frame_support::traits::ConstU32<1000>;
	type AssetId = MockAssetId;
	type AssetIdParameter = parity_scale_codec::Compact<MockAssetId>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<MockAccountId>>;
	type ForceOrigin = EnsureRoot<MockAccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = frame_support::traits::ConstU32<20>;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = AssetHelper;
	}
}

impl pallet_timestamp::Config for Test {
	type Moment = MockMomentResolution;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

pub const KEY_SEED: [u8; 32] = [0; 32];

parameter_types! {
	pub MockKeyPair: sp_core::sr25519::Pair = sp_core::sr25519::Pair::from_seed(&KEY_SEED);
}

pub const NULL_SIGNATURE: sp_core::sr25519::Signature = sp_core::sr25519::Signature([0; 64]);
pub const AUTHORITY_ID: MockAccountId = 37;

parameter_types! {
	pub const WildcardPalletId: PalletId = PalletId(*b"aj/wdcrd");
	pub const ChainId: u16 = CHAIN_ID;
	pub const NativeAssetId: MockAssetId = NATIVE_TOKEN_ID;
	pub const ChallengeBalance: MockBalance = 100;
}

pub type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, MockBlockNumber, MockCollectionId>;

pub struct MockOnMappingRequest;

impl OnMappingRequest<MockAssetId, MockCollectionId, MockItemId> for MockOnMappingRequest {
	fn on_fungible_asset_mapping(id: WideId) -> MockAssetId {
		MockAssetId::from_le_bytes([id[30], id[31], 0x00, 0x00]).saturated_into::<MockAssetId>()
	}

	fn on_non_fungible_collection_mapping(id: WideId) -> MockCollectionId {
		MockCollectionId::from_le_bytes([id[30], id[31], 0x00, 0x00])
			.saturated_into::<MockCollectionId>()
	}

	fn on_non_fungible_item_mapping(id: WideId) -> MockItemId {
		MockItemId::from_le_bytes([id[30], id[31], 0x00, 0x00]).saturated_into::<MockItemId>()
	}
}

impl pallet_wildcard::Config for Test {
	type PalletId = WildcardPalletId;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AssetId = MockAssetId;
	type Fungibles = Assets;
	type CollectionConfig = CollectionConfig;
	type ItemConfig = pallet_nfts::ItemConfig;
	type NonFungibles = Nft;
	type CollectionId = MockCollectionId;
	type ItemId = MockItemId;
	type OnMappingRequest = MockOnMappingRequest;
	type Time = Timestamp;
	type ChainId = ChainId;
	type NativeTokenAssetId = NativeAssetId;
	type ChallengeMinBalance = ChallengeBalance;
}

pub struct ExtBuilder {
	balances: Vec<(MockAccountId, MockBalance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: vec![(ALICE, 999_999_999), (BOB, 999_999_999), (CHARLIE, 999_999_999)] }
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext: sp_io::TestExternalities = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			StartTime::<Test>::put(0);
			EpochDuration::<Test>::put(SLOT_DURATION);
			SignerKey::<Test>::put(MockKeyPair::get().public());
		});
		ext
	}
}

pub fn create_collection(account: &MockAccountId) -> MockCollectionId {
	let _ = CurrencyOf::<Test>::deposit_creating(account, CollectionDeposit::get() + 999);
	let config = CollectionConfig::default();
	NftHelperOf::<Test>::create_collection(account, account, &config).unwrap()
}

pub fn mint_non_fungible(
	owner: &MockAccountId,
	collection_id: &MockCollectionId,
	item_id: &MockItemId,
) -> (MockCollectionId, MockItemId) {
	let _ = CurrencyOf::<Test>::deposit_creating(owner, ItemDeposit::get() + 999);
	let config = pallet_nfts::ItemConfig::default();
	NftHelperOf::<Test>::mint_into(collection_id, item_id, owner, &config, false).unwrap();
	(*collection_id, *item_id)
}

pub fn create_fungible(
	owner: MockAccountId,
	asset_id: MockAssetId,
	min_amount: MockBalance,
) -> MockAssetId {
	Assets::force_create(
		RuntimeOrigin::root(),
		parity_scale_codec::Compact(asset_id),
		owner,
		true,
		min_amount,
	)
	.expect("Should have created asset");

	asset_id
}

pub fn mint_fungible(
	owner: MockAccountId,
	asset_id: MockAssetId,
	amount: MockBalance,
) -> (MockAssetId, MockBalance) {
	Assets::mint(
		RuntimeOrigin::signed(owner),
		parity_scale_codec::Compact(asset_id),
		owner,
		amount,
	)
	.expect("Should hav minted asset");

	(asset_id, amount)
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			Wildcard::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		let block_number = System::block_number();

		Timestamp::set_timestamp(block_number * SLOT_DURATION);

		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Wildcard::on_initialize(System::block_number());
	}
}
