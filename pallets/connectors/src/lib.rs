// Copyright 2021 Centrifuge Foundation (centrifuge.io).
// This file is part of Centrifuge chain project.

// Centrifuge is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).

// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
#![cfg_attr(not(feature = "std"), no_std)]
use core::convert::TryFrom;

use cfg_traits::{
	connectors::{InboundQueue, OutboundQueue},
	PoolInspect,
};
use cfg_types::{
	domain_address::{Domain, DomainAddress},
	tokens::GeneralCurrencyIndex,
};
use cfg_utils::vec_to_fixed_array;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{
		fungibles::{Inspect, Mutate, Transfer},
		PalletInfo,
	},
	transactional,
};
use orml_traits::asset_registry::{self, Inspect as _};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Convert},
	FixedPointNumber, SaturatedConversion,
};
use sp_std::{convert::TryInto, vec, vec::Vec};
use xcm::{
	latest::NetworkId,
	prelude::{AccountKey20, GlobalConsensus, PalletInstance, X3},
	VersionedMultiLocation,
};

pub mod weights;

mod message;
pub use message::*;

mod routers;
pub use routers::*;

mod contract;
pub use contract::*;

mod inbound;

/// The Parachains that Centrifuge Connectors support.
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum ParachainId {
	/// Moonbeam - It may be Moonbeam on Polkadot, Moonriver on Kusama, or
	/// Moonbase on a testnet.
	Moonbeam,
}

// Type aliases
pub type PoolIdOf<T> = <<T as Config>::PoolInspect as PoolInspect<
	<T as frame_system::Config>::AccountId,
	CurrencyIdOf<T>,
>>::PoolId;

pub type TrancheIdOf<T> = <<T as Config>::PoolInspect as PoolInspect<
	<T as frame_system::Config>::AccountId,
	CurrencyIdOf<T>,
>>::TrancheId;

pub type MessageOf<T> =
	Message<Domain, PoolIdOf<T>, TrancheIdOf<T>, <T as Config>::Balance, <T as Config>::Rate>;

pub type CurrencyIdOf<T> = <T as Config>::CurrencyId;

pub type GeneralCurrencyIndexType = u128;

pub type GeneralCurrencyIndexOf<T> =
	GeneralCurrencyIndex<GeneralCurrencyIndexType, <T as pallet::Config>::GeneralCurrencyPrefix>;

#[frame_support::pallet]
pub mod pallet {
	use cfg_primitives::Moment;
	use cfg_traits::{
		CurrencyInspect, Investment, InvestmentCollector, Permissions, PoolInspect, TrancheCurrency,
	};
	use cfg_types::{
		permissions::{PermissionScope, PoolRole, Role},
		tokens::{ConnectorsWrappedToken, CustomMetadata},
	};
	use frame_support::{pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Zero;
	use xcm::latest::MultiLocation;

	use super::*;
	use crate::weights::WeightInfo;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// The source of truth for the balance of accounts in native currency.
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		/// The fixed point number representation for higher precision.
		type Rate: Parameter + Member + MaybeSerializeDeserialize + FixedPointNumber + TypeInfo;

		/// The origin allowed to make admin-like changes, such calling
		/// `set_domain_router`.
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The source of truth for pool inspection operations such as its
		/// existence, the corresponding tranche token or the investment
		/// currency.
		type PoolInspect: PoolInspect<Self::AccountId, CurrencyIdOf<Self>, Rate = Self::Rate>;

		/// The source of truth for investment permissions.
		type Permission: Permissions<
			Self::AccountId,
			Scope = PermissionScope<PoolIdOf<Self>, CurrencyIdOf<Self>>,
			Role = Role<TrancheIdOf<Self>, Moment>,
			Error = DispatchError,
		>;

		/// The UNIX timestamp provider type required for checking the validity
		/// of investments.
		type Time: UnixTime;

		/// The type for handling transfers, burning and minting of
		/// multi-assets.
		type Tokens: Mutate<Self::AccountId>
			+ Inspect<
				Self::AccountId,
				AssetId = CurrencyIdOf<Self>,
				Balance = <Self as pallet::Config>::Balance,
			> + Transfer<Self::AccountId>;

		/// The currency type of investments.
		type TrancheCurrency: TrancheCurrency<PoolIdOf<Self>, TrancheIdOf<Self>>
			+ Into<CurrencyIdOf<Self>>
			+ Clone;

		/// Enables investing and redeeming into investment classes.
		///
		/// NOTE: For the time being, `pallet_investments` serves as the
		/// implementor. However, eventually this should be provided by
		/// `pallet_foreign_investments`.
		type ForeignInvestment: Investment<
				Self::AccountId,
				Amount = <Self as Config>::Balance,
				CurrencyId = CurrencyIdOf<Self>,
				Error = DispatchError,
				InvestmentId = <Self as Config>::TrancheCurrency,
			> + InvestmentCollector<
				Self::AccountId,
				Error = DispatchError,
				InvestmentId = <Self as Config>::TrancheCurrency,
				Result = (),
			>;

		/// The source of truth for the transferability of assets via
		/// Connectors.
		type AssetRegistry: asset_registry::Inspect<
			AssetId = CurrencyIdOf<Self>,
			Balance = <Self as Config>::Balance,
			CustomMetadata = CustomMetadata,
		>;

		/// The currency type of transferrable tokens.
		type CurrencyId: Parameter
			+ Member
			+ Copy
			+ MaybeSerializeDeserialize
			+ Ord
			+ TypeInfo
			+ MaxEncodedLen
			+ TryInto<GeneralCurrencyIndexOf<Self>, Error = DispatchError>
			+ TryFrom<GeneralCurrencyIndexOf<Self>, Error = DispatchError>
			+ CurrencyInspect<CurrencyId = CurrencyIdOf<Self>>;

		/// The converter from a DomainAddress to a Substrate AccountId.
		type AccountConverter: Convert<DomainAddress, Self::AccountId>;

		/// The type for processing outgoing messages.
		type OutboundQueue: OutboundQueue<
			Sender = Self::AccountId,
			Message = MessageOf<Self>,
			Destination = Domain,
		>;

		/// The prefix for currencies added via Connectors.
		#[pallet::constant]
		type GeneralCurrencyPrefix: Get<[u8; 12]>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A message was sent to a domain
		MessageSent {
			message: MessageOf<T>,
			domain: Domain,
		},

		/// The Router for a given domain was set
		SetDomainRouter {
			domain: Domain,
			router: Router<CurrencyIdOf<T>>,
		},

		IncomingMessage {
			sender: T::AccountId,
			message: Vec<u8>,
		},
	}

	#[pallet::storage]
	pub(crate) type DomainRouter<T: Config> =
		StorageMap<_, Blake2_128Concat, Domain, Router<CurrencyIdOf<T>>>;

	/// The set of known connectors. This set is used as an allow-list when
	/// authorizing the origin of incoming messages through the `handle`
	/// extrinsic.
	#[pallet::storage]
	pub(crate) type KnownConnectors<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	#[pallet::error]
	pub enum Error<T> {
		/// Failed to map the asset to the corresponding Connector's General
		/// Index representation and thus cannot be used as an
		/// investment currency.
		AssetNotFound,
		/// The metadata of the given asset does not declare it as a pool
		/// currency and thus it cannot be used as an investment currency.
		AssetMetadataNotPoolCurrency,
		/// The metadata of the given asset does not declare it as transferable
		/// via connectors.
		AssetNotConnectorsTransferable,
		/// The asset is not a [ConnectorsWrappedToken] and thus cannot be
		/// transferred via connectors.
		AssetNotConnectorsWrappedToken,
		/// The given asset does not match the currency of the pool.
		AssetNotPoolCurrency,
		/// A pool could not be found.
		PoolNotFound,
		/// A tranche could not be found.
		TrancheNotFound,
		/// Could not find the metadata of a tranche token.
		TrancheMetadataNotFound,
		/// Failed to fetch a tranche token price.
		/// This can occur if `TrancheNotFound` or if effectively
		/// the price for this tranche has not yet been set.
		MissingTranchePrice,
		/// Router not set for a given domain.
		MissingRouter,
		/// Transfer amount must be non-zero.
		InvalidTransferAmount,
		/// A transfer to a non-whitelisted destination was attempted.
		UnauthorizedTransfer,
		/// Failed to build Ethereum_Xcm call.
		FailedToBuildEthereumXcmCall,
		/// The origin of an incoming message is not in the allow-list.
		InvalidIncomingMessageOrigin,
		/// Failed to decode an incoming message.
		InvalidIncomingMessage,
		/// The destination domain is invalid.
		InvalidDomain,
		/// The validity is in the past.
		InvalidTrancheInvestorValidity,
		/// Failed to match the provided GeneralCurrencyIndex against the
		/// investment currency of the pool.
		InvalidInvestCurrency,
		/// The currency is not allowed to be transferred via Connectors.
		InvalidTransferCurrency,
		/// The account derived from the [Domain] and [DomainAddress] has not
		/// been whitelisted as a TrancheInvestor.
		InvestorDomainAddressNotAMember,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as frame_system::Config>::AccountId: From<[u8; 32]>,
	{
		/// Set a Domain's router
		#[pallet::weight(< T as Config >::WeightInfo::set_domain_router())]
		#[pallet::call_index(0)]
		pub fn set_domain_router(
			origin: OriginFor<T>,
			domain: Domain,
			router: Router<CurrencyIdOf<T>>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin.clone())?;

			<DomainRouter<T>>::insert(domain.clone(), router.clone());
			Self::deposit_event(Event::SetDomainRouter { domain, router });

			Ok(())
		}

		/// Add an AccountId to the set of known connectors, allowing that
		/// origin to send incoming messages.
		#[pallet::weight(< T as Config >::WeightInfo::add_connector())]
		#[pallet::call_index(1)]
		pub fn add_connector(origin: OriginFor<T>, connector: T::AccountId) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			KnownConnectors::<T>::insert(connector, ());

			Ok(())
		}

		/// Add a pool to a given domain
		#[pallet::weight(< T as Config >::WeightInfo::add_pool())]
		#[pallet::call_index(2)]
		pub fn add_pool(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			domain: Domain,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(
				T::PoolInspect::pool_exists(pool_id),
				Error::<T>::PoolNotFound
			);

			T::OutboundQueue::submit(who, domain, Message::AddPool { pool_id })?;
			Ok(())
		}

		/// Add a tranche to a given domain
		#[pallet::weight(< T as Config >::WeightInfo::add_tranche())]
		#[pallet::call_index(3)]
		pub fn add_tranche(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			tranche_id: TrancheIdOf<T>,
			domain: Domain,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(
				T::PoolInspect::tranche_exists(pool_id, tranche_id),
				Error::<T>::TrancheNotFound
			);

			// Look up the metadata of the tranche token
			let currency_id = Self::derive_invest_id(pool_id, tranche_id)?;
			let metadata = T::AssetRegistry::metadata(&currency_id.into())
				.ok_or(Error::<T>::TrancheMetadataNotFound)?;
			let token_name = vec_to_fixed_array(metadata.name);
			let token_symbol = vec_to_fixed_array(metadata.symbol);
			let price = T::PoolInspect::get_tranche_token_price(pool_id, tranche_id)
				.ok_or(Error::<T>::MissingTranchePrice)?
				.price;

			// Send the message to the domain
			T::OutboundQueue::submit(
				who,
				domain,
				Message::AddTranche {
					pool_id,
					tranche_id,
					decimals: metadata.decimals.saturated_into(),
					token_name,
					token_symbol,
					price,
				},
			)?;

			Ok(())
		}

		/// Update the price of a tranche token
		#[pallet::weight(< T as Config >::WeightInfo::update_token_price())]
		#[pallet::call_index(4)]
		pub fn update_token_price(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			tranche_id: TrancheIdOf<T>,
			domain: Domain,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// TODO(follow-up PR): Move `get_tranche_token_price` to new trait.
			// https://centrifuge.hackmd.io/SERpps-URlG4hkOyyS94-w?both#fn-update_tranche_token_price
			let price = T::PoolInspect::get_tranche_token_price(pool_id, tranche_id)
				.ok_or(Error::<T>::MissingTranchePrice)?
				.price;

			T::OutboundQueue::submit(
				who,
				domain,
				Message::UpdateTrancheTokenPrice {
					pool_id,
					tranche_id,
					price,
				},
			)?;

			Ok(())
		}

		/// Update a member
		#[pallet::weight(< T as Config >::WeightInfo::update_member())]
		#[pallet::call_index(5)]
		pub fn update_member(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			tranche_id: TrancheIdOf<T>,
			domain_address: DomainAddress,
			valid_until: Moment,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(
				T::PoolInspect::pool_exists(pool_id),
				Error::<T>::PoolNotFound
			);
			ensure!(
				T::PoolInspect::tranche_exists(pool_id, tranche_id),
				Error::<T>::TrancheNotFound
			);
			ensure!(
				valid_until > Self::now(),
				Error::<T>::InvalidTrancheInvestorValidity
			);

			// Ensure that the destination address has been whitelisted as a TrancheInvestor
			// beforehand.
			ensure!(
				T::Permission::has(
					PermissionScope::Pool(pool_id),
					T::AccountConverter::convert(domain_address.clone()),
					Role::PoolRole(PoolRole::TrancheInvestor(tranche_id, valid_until))
				),
				Error::<T>::InvestorDomainAddressNotAMember
			);

			T::OutboundQueue::submit(
				who,
				domain_address.domain(),
				Message::UpdateMember {
					pool_id,
					tranche_id,
					valid_until,
					member: domain_address.address(),
				},
			)?;

			Ok(())
		}

		/// Transfer tranche tokens to a given address.
		///
		/// NOTE: Assumes `OutboundQueue` to check whether destination is local.
		///
		/// NOTE: The transferring account is not kept alive as we allow its
		/// death.
		#[pallet::weight(< T as Config >::WeightInfo::transfer())]
		#[pallet::call_index(6)]
		pub fn transfer_tranche_tokens(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			tranche_id: TrancheIdOf<T>,
			domain_address: DomainAddress,
			amount: <T as pallet::Config>::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!amount.is_zero(), Error::<T>::InvalidTransferAmount);
			ensure!(
				T::Permission::has(
					PermissionScope::Pool(pool_id),
					T::AccountConverter::convert(domain_address.clone()),
					Role::PoolRole(PoolRole::TrancheInvestor(tranche_id, Self::now()))
				),
				Error::<T>::UnauthorizedTransfer
			);

			// Ensure pool and tranche exist and derive invest id
			let invest_id = Self::derive_invest_id(pool_id, tranche_id)?;

			// Transfer to the domain account for bookkeeping
			T::Tokens::transfer(
				invest_id.into(),
				&who,
				&Domain::convert(domain_address.domain()),
				amount,
				// NOTE: Here, we allow death
				false,
			)?;

			T::OutboundQueue::submit(
				who.clone(),
				domain_address.domain(),
				Message::TransferTrancheTokens {
					pool_id,
					tranche_id,
					amount,
					domain: domain_address.domain(),
					sender: who
						.encode()
						.try_into()
						.map_err(|_| DispatchError::Other("Conversion to 32 bytes failed"))?,
					receiver: domain_address.address(),
				},
			)?;

			Ok(())
		}

		/// Transfer non-tranche tokens to a given address.
		///
		/// NOTE: Assumes `OutboundQueue` to check whether destination is local.
		///
		/// NOTE: The transferring account is not kept alive as we allow its
		/// death.
		#[pallet::weight(< T as Config >::WeightInfo::transfer())]
		#[pallet::call_index(7)]
		pub fn transfer(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			receiver: DomainAddress,
			amount: <T as pallet::Config>::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(!amount.is_zero(), Error::<T>::InvalidTransferAmount);
			ensure!(
				!CurrencyIdOf::<T>::is_tranche_token(currency_id),
				Error::<T>::InvalidTransferCurrency
			);
			let currency = Self::try_get_general_index(currency_id)?;

			// Check that the registered asset location matches the destination
			match Self::try_get_wrapped_token(&currency_id)? {
				ConnectorsWrappedToken::EVM { chain_id, .. } => {
					ensure!(
						Domain::EVM(chain_id) == receiver.domain(),
						Error::<T>::InvalidDomain
					);
				}
			}

			// Transfer to the domain account for bookkeeping
			T::Tokens::transfer(
				currency_id,
				&who,
				&Domain::convert(receiver.domain()),
				amount,
				// NOTE: Here, we allow death
				false,
			)?;

			T::OutboundQueue::submit(
				who.clone(),
				receiver.domain(),
				Message::Transfer {
					amount,
					currency,
					sender: who
						.encode()
						.try_into()
						.map_err(|_| DispatchError::Other("Conversion to 32 bytes failed"))?,
					receiver: receiver.address(),
				},
			)?;

			Ok(())
		}

		/// Add a currency to the set of known currencies on the domain derived
		/// from the given currency.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(8)]
		pub fn add_currency(origin: OriginFor<T>, currency_id: CurrencyIdOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let currency = Self::try_get_general_index(currency_id)?;

			let ConnectorsWrappedToken::EVM {
				chain_id,
				address: evm_address,
			} = Self::try_get_wrapped_token(&currency_id)?;

			T::OutboundQueue::submit(
				who,
				Domain::EVM(chain_id),
				Message::AddCurrency {
					currency,
					evm_address,
				},
			)?;

			Ok(())
		}

		/// Allow a currency to be used as a pool currency and to invest in a
		/// pool on the domain derived from the given currency.
		#[pallet::call_index(90)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn allow_pool_currency(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			tranche_id: TrancheIdOf<T>,
			currency_id: CurrencyIdOf<T>,
		) -> DispatchResult {
			// TODO(subsequent PR): In the future, should be permissioned by trait which
			// does not exist yet.
			// See spec: https://centrifuge.hackmd.io/SERpps-URlG4hkOyyS94-w?view#fn-add_pool_currency
			let who = ensure_signed(origin)?;

			// Ensure currency matches the currency of the pool
			let invest_id = Self::derive_invest_id(pool_id, tranche_id)?;
			ensure!(
				T::ForeignInvestment::accepted_payment_currency(invest_id, currency_id),
				Error::<T>::InvalidInvestCurrency
			);

			// Ensure the currency is enabled as pool_currency
			let metadata =
				T::AssetRegistry::metadata(&currency_id).ok_or(Error::<T>::AssetNotFound)?;
			ensure!(
				metadata.additional.pool_currency,
				Error::<T>::AssetMetadataNotPoolCurrency
			);

			// Derive GeneralIndex for currency
			let currency = Self::try_get_general_index(currency_id)?;

			let ConnectorsWrappedToken::EVM { chain_id, .. } =
				Self::try_get_wrapped_token(&currency_id)?;

			T::OutboundQueue::submit(
				who,
				Domain::EVM(chain_id),
				Message::AllowPoolCurrency { pool_id, currency },
			)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn now() -> Moment {
			T::Time::now().as_secs()
		}

		/// Returns the `u128` general index of a currency as the concatenation
		/// of the configured `GeneralCurrencyPrefix` and its local currency
		/// identifier.
		///
		/// Requires the currency to be registered in the `AssetRegistry`.
		///
		/// NOTE: Reverse operation of `try_get_currency_id`.
		pub fn try_get_general_index(currency: CurrencyIdOf<T>) -> Result<u128, DispatchError> {
			ensure!(
				T::AssetRegistry::metadata(&currency).is_some(),
				Error::<T>::AssetNotFound
			);

			let general_index: GeneralCurrencyIndexOf<T> = CurrencyIdOf::<T>::try_into(currency)?;

			Ok(general_index.index)
		}

		/// Returns the local currency identifier from from its general index.
		///
		/// Requires the currency to be registered in the `AssetRegistry`.
		///
		/// NOTE: Reverse operation of `try_get_general_index`.
		pub fn try_get_currency_id(
			index: GeneralCurrencyIndexOf<T>,
		) -> Result<CurrencyIdOf<T>, DispatchError> {
			let currency = CurrencyIdOf::<T>::try_from(index)?;
			ensure!(
				T::AssetRegistry::metadata(&currency).is_some(),
				Error::<T>::AssetNotFound
			);

			Ok(currency)
		}

		/// Checks whether the given currency is transferable via Connectors and
		/// whether its metadata contains a location which can be
		/// converted to [ConnectorsWrappedToken].
		///
		/// Requires the currency to be registered in the `AssetRegistry`.
		pub fn try_get_wrapped_token(
			currency_id: &CurrencyIdOf<T>,
		) -> Result<ConnectorsWrappedToken, DispatchError> {
			let meta = T::AssetRegistry::metadata(currency_id).ok_or(Error::<T>::AssetNotFound)?;
			ensure!(
				meta.additional.transferability.includes_connectors(),
				Error::<T>::AssetNotConnectorsTransferable
			);

			match meta.location {
				Some(VersionedMultiLocation::V3(MultiLocation {
					parents: 0,
					interior:
						X3(
							PalletInstance(pallet_instance),
							GlobalConsensus(NetworkId::Ethereum { chain_id }),
							AccountKey20 {
								network: None,
								key: address,
							},
						),
				})) if Some(pallet_instance.into())
					== <T as frame_system::Config>::PalletInfo::index::<Pallet<T>>() =>
				{
					Ok(ConnectorsWrappedToken::EVM { chain_id, address })
				}
				_ => Err(Error::<T>::AssetNotConnectorsWrappedToken.into()),
			}
		}

		/// Ensures that the given pool and tranche exists and returns the
		/// corresponding investment id.
		pub fn derive_invest_id(
			pool_id: PoolIdOf<T>,
			tranche_id: TrancheIdOf<T>,
		) -> Result<<T as pallet::Config>::TrancheCurrency, DispatchError> {
			ensure!(
				T::PoolInspect::pool_exists(pool_id),
				Error::<T>::PoolNotFound
			);
			ensure!(
				T::PoolInspect::tranche_exists(pool_id, tranche_id),
				Error::<T>::TrancheNotFound
			);

			Ok(TrancheCurrency::generate(pool_id, tranche_id))
		}

		/// Ensures that the payment currency of the given investment id matches
		/// the derived currency and returns the latter.
		pub fn try_get_payment_currency(
			invest_id: <T as pallet::Config>::TrancheCurrency,
			currency_index: GeneralCurrencyIndexOf<T>,
		) -> Result<CurrencyIdOf<T>, DispatchError> {
			// retrieve currency id from general index
			let currency = Self::try_get_currency_id(currency_index)?;

			ensure!(
				T::ForeignInvestment::accepted_payment_currency(invest_id, currency),
				Error::<T>::InvalidInvestCurrency
			);

			Ok(currency)
		}
	}

	impl<T: Config> InboundQueue for Pallet<T>
	where
		<T as frame_system::Config>::AccountId: From<[u8; 32]>,
	{
		type Message = MessageOf<T>;
		type Sender = DomainAddress;

		#[transactional]
		fn submit(sender: DomainAddress, msg: MessageOf<T>) -> DispatchResult {
			match msg {
				Message::Transfer {
					currency,
					receiver,
					amount,
					..
				} => Self::handle_transfer(currency.into(), receiver.into(), amount),
				Message::TransferTrancheTokens {
					pool_id,
					tranche_id,
					receiver,
					amount,
					..
				} => Self::handle_tranche_tokens_transfer(
					pool_id,
					tranche_id,
					sender,
					receiver.into(),
					amount,
				),
				Message::IncreaseInvestOrder {
					pool_id,
					tranche_id,
					investor,
					currency,
					amount,
				} => Self::handle_increase_invest_order(
					pool_id,
					tranche_id,
					investor.into(),
					currency.into(),
					amount,
				),
				Message::DecreaseInvestOrder {
					pool_id,
					tranche_id,
					investor,
					currency,
					amount,
				} => Self::handle_decrease_invest_order(
					pool_id,
					tranche_id,
					investor.into(),
					currency.into(),
					amount,
				),
				Message::IncreaseRedeemOrder {
					pool_id,
					tranche_id,
					investor,
					amount,
					..
				} => Self::handle_increase_redemption(
					pool_id,
					tranche_id,
					investor.into(),
					amount,
					sender,
				),
				Message::DecreaseRedeemOrder {
					pool_id,
					tranche_id,
					investor,
					currency,
					amount,
				} => Self::handle_decrease_redemption(
					pool_id,
					tranche_id,
					investor.into(),
					currency.into(),
					amount,
					sender,
				),
				Message::CollectInvest {
					pool_id,
					tranche_id,
					investor,
				} => Self::handle_collect_investment(pool_id, tranche_id, investor.into()),
				Message::CollectRedeem {
					pool_id,
					tranche_id,
					investor,
				} => Self::handle_collect_redemption(pool_id, tranche_id, investor.into()),
				_ => Err(Error::<T>::InvalidIncomingMessage.into()),
			}?;

			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use codec::{Decode, Encode};

	use crate::Domain;

	#[test]
	fn test_domain_encode_decode() {
		test_domain_identity(Domain::Centrifuge);
		test_domain_identity(Domain::EVM(1284));
		test_domain_identity(Domain::EVM(1));
	}

	/// Test that decode . encode results in the original value
	fn test_domain_identity(domain: Domain) {
		let encoded = domain.encode();
		let decoded: Domain = Domain::decode(&mut encoded.as_slice()).expect("");

		assert_eq!(domain, decoded);
	}
}
