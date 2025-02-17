#![no_std]

use multiversx_sc::derive_imports::*;
#[allow(unused_imports)]
use multiversx_sc::imports::*;

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone, Copy)]
pub enum Status {
    FundingPeriod,
    Successful,
    Failed,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait CrowdfundingSc {
    #[init]
    fn init(&self, target: BigUint, deadline: u64, min_fund: BigUint, max_deposit_per_donor: BigUint, max_target: BigUint) {
        require!(target > 0, "Target must be more than 0");

        // Validació dels nous paràmetres que tenim en compte pel crowdfounding+.
        require!(max_target >= target, "Max target must be greather or equals than target.");
        require!(min_fund > 0, "Min fund must be greater than 0");
        require!(max_deposit_per_donor >= min_fund, "Max fund per donor must be greater or equal than min fund.");
        self.target().set(target);
        self.min_fund().set(min_fund);
        self.max_deposit_per_donor().set(max_deposit_per_donor);
        self.max_target().set(max_target);

        require!(
            deadline > self.get_current_time(),
            "Deadline can't be in the past"
        );
        self.deadline().set(deadline);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint]
    #[payable("EGLD")]
    fn fund(&self) {
        let payment = self.call_value().egld_value().clone_value();

        // Validació perquè el donatiu tingui el mínim establert.
        require!(
            payment >= self.min_fund().get(),
            "Fund doesn't reach the minimum accepted."
        );

        // Validació perquè el donatiu no faci superar el target màxim.
        require!(
            self.get_current_funds() + payment.clone() <= self.max_target().get(),
            "Fund would exceed the maximum amount."
        );

        let current_time = self.blockchain().get_block_timestamp();
        require!(
            current_time < self.deadline().get(),
            "cannot fund after deadline"
        );

        // Validació del donatiu màxim per donant.
        let caller = self.blockchain().get_caller();
        let deposited_amount = self.deposit(&caller).get();
        let wallet_deposit = deposited_amount + payment;
        if self.max_deposit_per_donor().get() > 0u32 {
            require!(
                wallet_deposit <= self.max_deposit_per_donor().get(),
                "Deposit exceeds maximum allowed!"
            );
        }

        self.deposit(&caller).set(wallet_deposit);
    }

    #[endpoint]
    fn claim(&self) {
        match self.status() {
            Status::FundingPeriod => sc_panic!("cannot claim before deadline"),
            Status::Successful => {
                let caller = self.blockchain().get_caller();
                require!(
                    caller == self.blockchain().get_owner_address(),
                    "only owner can claim successful funding"
                );

                let sc_balance = self.get_current_funds();
                self.send().direct_egld(&caller, &sc_balance);
            }
            Status::Failed => {
                let caller = self.blockchain().get_caller();
                let deposit = self.deposit(&caller).get();

                if deposit > 0u32 {
                    self.deposit(&caller).clear();
                    self.send().direct_egld(&caller, &deposit);
                }
            }
        }
    }

    #[only_owner]
    #[endpoint(setMaxDepositPerWallet)]
    fn set_max_deposit_per_donor(&self, max_deposit: BigUint) {
        self.max_deposit_per_donor().set(max_deposit);
    }

    #[view]
    fn status(&self) -> Status {
        if self.get_current_time() <= self.deadline().get() {
            Status::FundingPeriod
        } else if self.get_current_funds() >= self.target().get() {
            Status::Successful
        } else {
            Status::Failed
        }
    }

    #[view(getCurrentFunds)]
    fn get_current_funds(&self) -> BigUint {
        self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0)
    }

    // private

    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }

    // storage

    #[view(getTarget)]
    #[storage_mapper("target")]
    fn target(&self) -> SingleValueMapper<BigUint>;

    #[view(getDeadline)]
    #[storage_mapper("deadline")]
    fn deadline(&self) -> SingleValueMapper<u64>;

    #[view(getMinFund)]
    #[storage_mapper("min_fund")]
    fn min_fund(&self) -> SingleValueMapper<BigUint>;

    #[view(getMaxDepositPerDonor)]
    #[storage_mapper("max_deposit_per_donor")]
    fn max_deposit_per_donor(&self) -> SingleValueMapper<BigUint>;

    #[view(getMaxTarget)]
    #[storage_mapper("max_target")]
    fn max_target(&self) -> SingleValueMapper<BigUint>;

    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, donor: &ManagedAddress) -> SingleValueMapper<BigUint>;

    
}
