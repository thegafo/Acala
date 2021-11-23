// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Unit tests using a mock with no fees.
//! This is mainly used to test economic model.

#![cfg(test)]

use super::*;
use frame_support::assert_ok;
use mock_no_fees::{
	dollar, AccountId, Currencies, Event, ExtBuilder, HomaLite, NoFeeRuntime, Origin, System, ALICE, BOB, DAVE, KSM,
	LKSM,
};

#[test]
fn no_fee_runtime_has_no_fees() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));

		// Mint costs no fees
		assert_ok!(HomaLite::mint(Origin::signed(ALICE), dollar(1_000)));
		assert_eq!(
			HomaLite::get_exchange_rate(),
			ExchangeRate::saturating_from_rational(1, 10)
		);
		System::assert_last_event(Event::HomaLite(crate::Event::Minted(
			ALICE,
			dollar(1_000),
			dollar(10_000),
		)));
		assert_eq!(Currencies::free_balance(KSM, &ALICE), dollar(999_000));
		assert_eq!(Currencies::free_balance(LKSM, &ALICE), dollar(10_000));

		assert_ok!(HomaLite::mint(Origin::signed(BOB), dollar(5_000)));
		System::assert_last_event(Event::HomaLite(crate::Event::Minted(
			BOB,
			dollar(5_000),
			dollar(50_000),
		)));
		assert_eq!(Currencies::free_balance(KSM, &BOB), dollar(995_000));
		assert_eq!(Currencies::free_balance(LKSM, &BOB), dollar(50_000));

		//Redeem costs no fees
		assert_ok!(HomaLite::request_redeem(
			Origin::signed(BOB),
			dollar(50_000),
			Permill::zero()
		));
		System::assert_last_event(Event::HomaLite(crate::Event::RedeemRequested(
			BOB,
			dollar(50_000),
			Permill::zero(),
			0,
		)));
		assert_ok!(HomaLite::mint(Origin::signed(ALICE), dollar(5_000)));

		assert_eq!(Currencies::free_balance(KSM, &ALICE), dollar(994_000));
		assert_eq!(Currencies::free_balance(LKSM, &ALICE), dollar(60_000));
		assert_eq!(Currencies::free_balance(KSM, &BOB), dollar(1_000_000));
		assert_eq!(Currencies::free_balance(LKSM, &BOB), 0);
		let events = System::events();
		assert_eq!(
			events[events.len() - 2].event,
			Event::HomaLite(crate::Event::Redeemed(BOB, dollar(5000), dollar(50000),))
		);
		assert_eq!(
			events[events.len() - 1].event,
			Event::HomaLite(crate::Event::Minted(ALICE, dollar(5000), dollar(50000),))
		);

		// Redeem from AvailableStakingBalance costs no fees
		assert_ok!(HomaLite::schedule_unbond(Origin::root(), dollar(50_000), 0));
		let _ = HomaLite::on_idle(0, 5_000_000_000);

		assert_ok!(HomaLite::request_redeem(
			Origin::signed(DAVE),
			dollar(100_000),
			Permill::zero()
		));

		assert_eq!(HomaLite::available_staking_balance(), dollar(40_000));
		assert_eq!(Currencies::free_balance(KSM, &DAVE), dollar(10_000));
		assert_eq!(Currencies::free_balance(LKSM, &DAVE), dollar(900_000));

		let events = System::events();
		assert_eq!(
			events[events.len() - 9].event,
			Event::HomaLite(crate::Event::ScheduledUnbondAdded(dollar(50_000), 0))
		);
		assert_eq!(
			events[events.len() - 8].event,
			Event::HomaLite(crate::Event::ScheduledUnbondWithdrew(dollar(50_000)))
		);
		assert_eq!(
			events[events.len() - 7].event,
			Event::Tokens(orml_tokens::Event::Reserved(LKSM, DAVE, dollar(100_000)))
		);
		assert_eq!(
			events[events.len() - 6].event,
			Event::HomaLite(crate::Event::RedeemRequested(DAVE, dollar(100_000), Permill::zero(), 0))
		);
		assert_eq!(
			events[events.len() - 5].event,
			Event::HomaLite(crate::Event::TotalStakingCurrencySet(dollar(96_000)))
		);
		assert_eq!(
			events[events.len() - 4].event,
			Event::Tokens(orml_tokens::Event::Endowed(KSM, DAVE, dollar(10_000)))
		);
		assert_eq!(
			events[events.len() - 3].event,
			Event::Currencies(module_currencies::Event::Deposited(KSM, DAVE, dollar(10_000)))
		);
		assert_eq!(
			events[events.len() - 2].event,
			Event::Tokens(orml_tokens::Event::Unreserved(LKSM, DAVE, dollar(100_000)))
		);
		assert_eq!(
			events[events.len() - 1].event,
			Event::HomaLite(crate::Event::Redeemed(DAVE, dollar(10_000), dollar(100_000)))
		);
	});
}

#[test]
fn mint_with_xcm_does_not_change_exchange_rate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));

		let exchange_rate = HomaLite::get_exchange_rate();

		for _ in 0..100 {
			assert_ok!(HomaLite::mint(Origin::signed(ALICE), dollar(500)));
			assert_eq!(exchange_rate, HomaLite::get_exchange_rate());
		}

		assert_eq!(Currencies::free_balance(KSM, &ALICE), dollar(950_000));
		assert_eq!(Currencies::free_balance(LKSM, &ALICE), dollar(500_000));
	});
}

#[test]
fn mint_with_redeem_does_not_change_exchange_rate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));
		assert_ok!(HomaLite::request_redeem(
			Origin::signed(DAVE),
			dollar(1_000_000),
			Permill::zero()
		));
		let exchange_rate = HomaLite::get_exchange_rate();

		for _ in 0..100 {
			assert_ok!(HomaLite::mint(Origin::signed(ALICE), dollar(500)));
			assert_eq!(exchange_rate, HomaLite::get_exchange_rate());
		}

		assert_eq!(Currencies::free_balance(KSM, &ALICE), dollar(950_000));
		assert_eq!(Currencies::free_balance(LKSM, &ALICE), dollar(500_000));

		assert_eq!(Currencies::free_balance(KSM, &DAVE), dollar(50_000));
		assert_eq!(Currencies::free_balance(LKSM, &DAVE), 0);
		assert_eq!(Currencies::reserved_balance(LKSM, &DAVE), dollar(500_000));

		// Add redeem with 50% extra reward.
		assert_ok!(HomaLite::request_redeem(
			Origin::signed(ALICE),
			dollar(500_000),
			Permill::from_percent(50)
		));

		for _ in 0..100 {
			assert_ok!(HomaLite::mint(Origin::signed(BOB), dollar(1_000)));
			assert_eq!(exchange_rate, HomaLite::get_exchange_rate());
		}

		// 950_000 + 50_000 * 50%, since the other 50% went to the minter as rewards.
		assert_eq!(Currencies::free_balance(KSM, &ALICE), dollar(975_000));
		assert_eq!(Currencies::free_balance(LKSM, &ALICE), 0);

		// Got 25_000 extra as extra rewards
		assert_eq!(Currencies::free_balance(KSM, &BOB), dollar(925_000));
		assert_eq!(Currencies::free_balance(LKSM, &BOB), dollar(1_000_000));

		assert_eq!(Currencies::free_balance(KSM, &DAVE), dollar(100_000));
		assert_eq!(Currencies::free_balance(LKSM, &DAVE), 0);
		assert_eq!(Currencies::reserved_balance(LKSM, &DAVE), 0);
	});
}

#[test]
fn redeem_with_available_staking_does_not_change_exchange_rate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));

		assert_ok!(HomaLite::adjust_available_staking_balance(
			Origin::root(),
			dollar(100) as i128,
			100
		));

		let exchange_rate = HomaLite::get_exchange_rate();

		// test repeated redeem using available staking
		for _ in 0..100 {
			assert_ok!(HomaLite::request_redeem(
				Origin::signed(DAVE),
				dollar(10),
				Permill::zero()
			));
			assert_eq!(exchange_rate, HomaLite::get_exchange_rate());
		}

		assert_eq!(HomaLite::available_staking_balance(), 0);
		assert_eq!(Currencies::free_balance(KSM, &DAVE), dollar(100));
		assert_eq!(Currencies::free_balance(LKSM, &DAVE), dollar(999_000));
		assert_eq!(Currencies::reserved_balance(LKSM, &DAVE), 0);

		// Test repeated adjust_available_staking_balance with a queued redeem request.
		assert_ok!(HomaLite::request_redeem(
			Origin::signed(DAVE),
			dollar(10_000),
			Permill::zero()
		));
		for _ in 0..100 {
			assert_ok!(HomaLite::adjust_available_staking_balance(
				Origin::root(),
				dollar(10) as i128,
				100
			));
			assert_eq!(exchange_rate, HomaLite::get_exchange_rate());
		}

		assert_eq!(HomaLite::available_staking_balance(), 0);
		assert_eq!(Currencies::free_balance(KSM, &DAVE), dollar(1100));
		assert_eq!(Currencies::free_balance(LKSM, &DAVE), dollar(989_000));
		assert_eq!(Currencies::reserved_balance(LKSM, &DAVE), 0);
	});
}

#[test]
fn mint_and_redeem_at_the_same_time_does_not_change_exchange_rate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));
		assert_ok!(HomaLite::adjust_available_staking_balance(
			Origin::root(),
			dollar(5_000) as i128,
			0
		));

		let exchange_rate = HomaLite::get_exchange_rate();

		// The first 50 redeems are done using available_staking_balance.
		// The next 50 redeems are matched with mint.
		for _ in 0..100 {
			assert_ok!(HomaLite::request_redeem(
				Origin::signed(DAVE),
				dollar(1000),
				Permill::zero()
			));
			assert_ok!(HomaLite::mint(Origin::signed(ALICE), dollar(100)));
			assert_eq!(exchange_rate, HomaLite::get_exchange_rate());
		}

		assert_eq!(Currencies::free_balance(KSM, &ALICE), dollar(990_000));
		assert_eq!(Currencies::free_balance(LKSM, &ALICE), dollar(100_000));

		assert_eq!(Currencies::free_balance(KSM, &DAVE), dollar(10_000));
		assert_eq!(Currencies::free_balance(LKSM, &DAVE), dollar(900_000));
	});
}

#[test]
fn updating_and_cancelling_redeem_requests_does_not_change_exchange_rate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));

		let exchange_rate = HomaLite::get_exchange_rate();

		for i in 1..101 {
			assert_ok!(HomaLite::request_redeem(
				Origin::signed(DAVE),
				dollar(i * 100u128),
				Permill::from_percent(i as u32)
			));
			assert_eq!(exchange_rate, HomaLite::get_exchange_rate());
		}
		assert_eq!(HomaLite::redeem_requests(DAVE), Some((dollar(10_000), Permill::one())));

		for i in 1..101 {
			assert_ok!(HomaLite::request_redeem(
				Origin::signed(DAVE),
				dollar((100 - i) * 100u128),
				Permill::from_percent(100 - i as u32)
			));
			assert_eq!(exchange_rate, HomaLite::get_exchange_rate());
		}
		assert_eq!(HomaLite::redeem_requests(DAVE), None);

		assert_eq!(Currencies::free_balance(KSM, &DAVE), 0);
		assert_eq!(Currencies::free_balance(LKSM, &DAVE), dollar(1_000_000));
	});
}

#[test]
fn mint_match_from_previous_redeem_requests() {
	ExtBuilder::empty().build().execute_with(|| {
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));

		for i in 0..10 {
			let account = AccountId::from([i as u8; 32]);
			assert_ok!(Currencies::update_balance(
				Origin::root(),
				account.clone(),
				LKSM,
				dollar(1000 as u128) as i128
			));
			assert_ok!(HomaLite::request_redeem(
				Origin::signed(account),
				dollar(1000),
				Permill::zero()
			));
		}

		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));

		// This is the default order the redeem requests are iterated.
		let mut default_order = vec![];
		for (redeemer, _) in RedeemRequests::<NoFeeRuntime>::iter() {
			default_order.push(redeemer);
		}
		assert_eq!(
			default_order,
			vec![
				AccountId::from([1u8; 32]),
				AccountId::from([6u8; 32]),
				AccountId::from([2u8; 32]),
				AccountId::from([3u8; 32]),
				AccountId::from([8u8; 32]),
				AccountId::from([9u8; 32]),
				AccountId::from([7u8; 32]),
				AccountId::from([4u8; 32]),
				AccountId::from([5u8; 32]),
				AccountId::from([0u8; 32]),
			]
		);

		let minter = AccountId::from([255u8; 32]);
		assert_ok!(Currencies::update_balance(
			Origin::root(),
			minter.clone(),
			KSM,
			dollar(100 as u128) as i128
		));

		// If unset, `LastRedeemRequestKeyIterated` should be the default account Id
		assert!(HomaLite::last_redeem_request_key_iterated().is_empty());

		// Minting once for each item in redeem request should be iterated once
		for i in 0..10 {
			assert_ok!(HomaLite::mint(Origin::signed(minter.clone()), dollar(10)));
			// Each item should be iterated once
			assert_eq!(
				HomaLite::redeem_requests(default_order[i].clone()),
				Some((dollar(900), Permill::zero()))
			);
			assert_eq!(Currencies::free_balance(KSM, &default_order[i]), dollar(10));
			// Ensure `LastRedeemRequestKeyIterated` is setup correctly.
			assert_eq!(
				HomaLite::last_redeem_request_key_iterated(),
				RedeemRequests::<NoFeeRuntime>::hashed_key_for(default_order[i].clone())
			);
		}

		// Check mint operations are successful.
		assert_eq!(Currencies::free_balance(KSM, &minter), 0);
		assert_eq!(Currencies::free_balance(LKSM, &minter), dollar(1000));

		// Test iterate only wrap around once without double-redeem.
		assert_ok!(Currencies::update_balance(
			Origin::root(),
			minter.clone(),
			KSM,
			dollar(1000 as u128) as i128
		));

		assert_eq!(HomaLite::total_staking_currency(), dollar(1000));

		// 900 should be minted from redeem requests, 100 from XCM.
		assert_ok!(HomaLite::mint(Origin::signed(minter.clone()), dollar(1000)));

		// All redeem requests should be fulfilled, and only once.
		for i in 0..10 {
			assert_eq!(HomaLite::redeem_requests(default_order[i].clone()), None);
			assert_eq!(Currencies::free_balance(KSM, &default_order[i]), dollar(100));
			assert_eq!(Currencies::free_balance(LKSM, &default_order[i]), 0);
			assert_eq!(Currencies::reserved_balance(LKSM, &default_order[i]), 0);
		}

		assert_eq!(Currencies::free_balance(KSM, &minter), 0);
		assert_eq!(Currencies::free_balance(LKSM, &minter), dollar(11000));

		// 100 KSM redeemed from XCM, increasing the staking total.
		assert_eq!(HomaLite::total_staking_currency(), dollar(1100));
	});
}

#[test]
fn unbonded_staking_match_from_previous_redeem_requests() {
	let mut unbond = |amount: Balance| -> DispatchResult {
		assert_ok!(HomaLite::schedule_unbond(Origin::root(), amount, 0));
		HomaLite::on_idle(0, 5_000_000_000);
		Ok(())
	};

	let mut adjust_available_staking_balance = |amount: Balance| -> DispatchResult {
		HomaLite::adjust_available_staking_balance(Origin::root(), amount as i128, 1_000)
	};

	// Test unbonding can iterate from `LastRedeemRequestKeyIterated`
	test_increase_staking_match_from_previous_redeem_requests(&mut unbond);

	// Test `adjust_available_staking_balance` can iterate from `LastRedeemRequestKeyIterated`
	test_increase_staking_match_from_previous_redeem_requests(&mut adjust_available_staking_balance);
}

// Helper function that tests when increasing Staking currency, the redeem requests are processed
// from the `LastRedeemRequestKeyIterated`. Takes a Function that increases the StakingCurrency and
// matches redeem requests.
fn test_increase_staking_match_from_previous_redeem_requests(
	mut increase_staking: impl FnMut(Balance) -> DispatchResult,
) {
	ExtBuilder::empty().build().execute_with(|| {
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));

		// Give someone extra fund so total staking does not reduce to zero.
		assert_ok!(Currencies::update_balance(
			Origin::root(),
			AccountId::from([255u8; 32]),
			LKSM,
			dollar(10 as u128) as i128
		));

		for i in 0..10 {
			let account = AccountId::from([i as u8; 32]);
			assert_ok!(Currencies::update_balance(
				Origin::root(),
				account.clone(),
				LKSM,
				dollar(1000 as u128) as i128
			));
			assert_ok!(HomaLite::request_redeem(
				Origin::signed(account),
				dollar(1000),
				Permill::zero()
			));
		}

		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));

		// This is the default order the redeem requests are iterated.
		let mut default_order = vec![];
		for (redeemer, _) in RedeemRequests::<NoFeeRuntime>::iter() {
			default_order.push(redeemer);
		}
		assert_eq!(
			default_order,
			vec![
				AccountId::from([1u8; 32]),
				AccountId::from([6u8; 32]),
				AccountId::from([2u8; 32]),
				AccountId::from([3u8; 32]),
				AccountId::from([8u8; 32]),
				AccountId::from([9u8; 32]),
				AccountId::from([7u8; 32]),
				AccountId::from([4u8; 32]),
				AccountId::from([5u8; 32]),
				AccountId::from([0u8; 32]),
			]
		);

		// If unset, `LastRedeemRequestKeyIterated` should be the default account Id
		assert!(HomaLite::last_redeem_request_key_iterated().is_empty());

		assert_eq!(HomaLite::total_staking_currency(), dollar(1001));

		// Minting once for each item in redeem request should be iterated once
		for i in 0..10 {
			assert_ok!(increase_staking(dollar(10)));
			assert_eq!(HomaLite::total_staking_currency(), dollar(1001 - (i as u128 + 1) * 10));
			// Each item should be iterated once
			assert_eq!(
				HomaLite::redeem_requests(default_order[i].clone()),
				Some((dollar(900), Permill::zero()))
			);
			assert_eq!(Currencies::free_balance(KSM, &default_order[i]), dollar(10));
			// Ensure `LastRedeemRequestKeyIterated` is setup correctly.
			assert_eq!(
				HomaLite::last_redeem_request_key_iterated(),
				RedeemRequests::<NoFeeRuntime>::hashed_key_for(default_order[i].clone())
			);
		}

		// Ensure `LastRedeemRequestKeyIterated` is setup correctly.
		assert_eq!(
			HomaLite::last_redeem_request_key_iterated(),
			RedeemRequests::<NoFeeRuntime>::hashed_key_for(AccountId::default().clone())
		);

		assert_eq!(HomaLite::total_staking_currency(), dollar(901));

		// Test iterate only wrap around once without double-redeem.
		// 900 should be used to clear all redeem requests, 100 is then left over.
		assert_ok!(increase_staking(dollar(1000)));

		// All redeem requests should be fulfilled, and only once.
		for i in 0..10 {
			assert_eq!(HomaLite::redeem_requests(default_order[i].clone()), None);
			assert_eq!(Currencies::free_balance(KSM, &default_order[i]), dollar(100));
			assert_eq!(Currencies::free_balance(LKSM, &default_order[i]), 0);
			assert_eq!(Currencies::reserved_balance(LKSM, &default_order[i]), 0);
		}

		assert_eq!(HomaLite::total_staking_currency(), dollar(1));
		assert_eq!(HomaLite::available_staking_balance(), dollar(100));
	});
}

#[test]
fn redeem_does_not_restart_if_previous_key_is_removed() {
	ExtBuilder::empty().build().execute_with(|| {
		assert_ok!(HomaLite::set_minting_cap(Origin::root(), dollar(1_000_000)));

		for i in 0..5 {
			let account = AccountId::from([i as u8; 32]);
			assert_ok!(Currencies::update_balance(
				Origin::root(),
				account.clone(),
				LKSM,
				dollar(1000 as u128) as i128
			));
			assert_ok!(HomaLite::request_redeem(
				Origin::signed(account),
				dollar(1000),
				Permill::zero()
			));
		}

		assert_ok!(HomaLite::set_total_staking_currency(
			Origin::root(),
			Currencies::total_issuance(LKSM) / 10
		));

		// This is the default order the redeem requests are iterated.
		let mut default_order = vec![];
		for (redeemer, _) in RedeemRequests::<NoFeeRuntime>::iter() {
			default_order.push(redeemer);
		}
		assert_eq!(
			default_order,
			vec![
				AccountId::from([1u8; 32]),
				AccountId::from([2u8; 32]),
				AccountId::from([3u8; 32]),
				AccountId::from([4u8; 32]),
				AccountId::from([0u8; 32]),
			]
		);

		let minter = AccountId::from([255u8; 32]);
		assert_ok!(Currencies::update_balance(
			Origin::root(),
			minter.clone(),
			KSM,
			dollar(100 as u128) as i128
		));

		// Mint from the first element in the iterator
		assert_ok!(HomaLite::mint(Origin::signed(minter.clone()), dollar(10)));
		assert_eq!(
			HomaLite::redeem_requests(AccountId::from([1u8; 32])),
			Some((dollar(900), Permill::zero()))
		);

		assert_eq!(Currencies::free_balance(KSM, &AccountId::from([1u8; 32])), dollar(10));
		assert_eq!(
			HomaLite::last_redeem_request_key_iterated(),
			RedeemRequests::<NoFeeRuntime>::hashed_key_for(AccountId::from([1u8; 32]))
		);

		// Remove the next element
		RedeemRequests::<NoFeeRuntime>::remove(AccountId::from([2u8; 32]));
		assert_eq!(HomaLite::redeem_requests(AccountId::from([2u8; 32])), None);

		// Next mint should continue without restarting
		assert_ok!(HomaLite::mint(Origin::signed(minter.clone()), dollar(10)));
		assert_eq!(
			HomaLite::redeem_requests(AccountId::from([3u8; 32])),
			Some((dollar(900), Permill::zero()))
		);

		assert_eq!(Currencies::free_balance(KSM, &AccountId::from([3u8; 32])), dollar(10));
		assert_eq!(
			HomaLite::last_redeem_request_key_iterated(),
			RedeemRequests::<NoFeeRuntime>::hashed_key_for(AccountId::from([3u8; 32]))
		);

		// remove the last final 2 elements
		RedeemRequests::<NoFeeRuntime>::remove(AccountId::from([4u8; 32]));
		RedeemRequests::<NoFeeRuntime>::remove(AccountId::from([0u8; 32]));

		// Next mint should start from the beginning
		assert_ok!(HomaLite::mint(Origin::signed(minter.clone()), dollar(10)));
		assert_eq!(
			HomaLite::redeem_requests(AccountId::from([1u8; 32])),
			Some((dollar(800), Permill::zero()))
		);

		assert_eq!(Currencies::free_balance(KSM, &AccountId::from([1u8; 32])), dollar(20));
		assert_eq!(
			HomaLite::last_redeem_request_key_iterated(),
			RedeemRequests::<NoFeeRuntime>::hashed_key_for(AccountId::from([1u8; 32]))
		);
	});
}
