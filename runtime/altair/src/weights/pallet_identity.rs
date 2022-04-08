//! Autogenerated weights for pallet_identity
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-21, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("altair-dev"), DB CACHE: 1024

// Executed Command:
// target/release/centrifuge-chain
// benchmark
// --chain=altair-dev
// --steps=50
// --repeat=20
// --pallet=pallet_identity
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=/tmp/runtime/altair/src/weights/pallet_identity.rs
// --template=./scripts/runtime-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use pallet_identity::weights::WeightInfo;
use sp_std::marker::PhantomData;

/// Weights for pallet_identity using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn add_registrar(r: u32) -> Weight {
		(29_686_000 as Weight) // Standard Error: 17_000
			.saturating_add((403_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_identity(r: u32, x: u32) -> Weight {
		(63_926_000 as Weight) // Standard Error: 26_000
			.saturating_add((526_000 as Weight).saturating_mul(r as Weight)) // Standard Error: 3_000
			.saturating_add((883_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_subs_new(s: u32) -> Weight {
		(53_895_000 as Weight) // Standard Error: 7_000
			.saturating_add((7_087_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(s as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	fn set_subs_old(p: u32) -> Weight {
		(52_595_000 as Weight) // Standard Error: 3_000
			.saturating_add((2_293_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
	}
	fn clear_identity(r: u32, s: u32, x: u32) -> Weight {
		(68_736_000 as Weight) // Standard Error: 42_000
			.saturating_add((560_000 as Weight).saturating_mul(r as Weight)) // Standard Error: 5_000
			.saturating_add((2_229_000 as Weight).saturating_mul(s as Weight)) // Standard Error: 5_000
			.saturating_add((542_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	fn request_judgement(r: u32, x: u32) -> Weight {
		(65_722_000 as Weight) // Standard Error: 26_000
			.saturating_add((692_000 as Weight).saturating_mul(r as Weight)) // Standard Error: 3_000
			.saturating_add((962_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn cancel_request(r: u32, x: u32) -> Weight {
		(60_802_000 as Weight) // Standard Error: 22_000
			.saturating_add((484_000 as Weight).saturating_mul(r as Weight)) // Standard Error: 2_000
			.saturating_add((944_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_fee(r: u32) -> Weight {
		(11_474_000 as Weight) // Standard Error: 7_000
			.saturating_add((352_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_account_id(r: u32) -> Weight {
		(11_020_000 as Weight) // Standard Error: 10_000
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_fields(r: u32) -> Weight {
		(10_553_000 as Weight) // Standard Error: 7_000
			.saturating_add((404_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn provide_judgement(r: u32, x: u32) -> Weight {
		(45_142_000 as Weight) // Standard Error: 23_000
			.saturating_add((558_000 as Weight).saturating_mul(r as Weight)) // Standard Error: 2_000
			.saturating_add((951_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn kill_identity(r: u32, s: u32, x: u32) -> Weight {
		(88_994_000 as Weight) // Standard Error: 43_000
			.saturating_add((709_000 as Weight).saturating_mul(r as Weight)) // Standard Error: 5_000
			.saturating_add((2_322_000 as Weight).saturating_mul(s as Weight)) // Standard Error: 5_000
			.saturating_add((8_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	fn add_sub(s: u32) -> Weight {
		(71_223_000 as Weight) // Standard Error: 3_000
			.saturating_add((319_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn rename_sub(s: u32) -> Weight {
		(22_173_000 as Weight) // Standard Error: 1_000
			.saturating_add((107_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_sub(s: u32) -> Weight {
		(72_295_000 as Weight) // Standard Error: 3_000
			.saturating_add((306_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn quit_sub(s: u32) -> Weight {
		(48_367_000 as Weight) // Standard Error: 3_000
			.saturating_add((298_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}