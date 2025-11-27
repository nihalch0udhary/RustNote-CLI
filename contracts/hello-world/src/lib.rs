#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec,
};

const TENDER_NS: Symbol = symbol_short!("TENDR");

#[contracttype]
#[derive(Clone)]
pub enum TenderStatus {
    Open,
    Awarded,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct Tender {
    pub tender_id: u64,
    pub authority: Address,
    pub title: String,
    pub status: TenderStatus,
    pub winning_bid: Option<u64>, // bid_id
}

#[contracttype]
#[derive(Clone)]
pub struct Bid {
    pub bid_id: u64,
    pub tender_id: u64,
    pub vendor: Address,
    pub amount: i128,
    pub details: String,
}

#[contract]
pub struct SecureTenderHub;

#[contractimpl]
impl SecureTenderHub {
    // Authority creates a new tender
    pub fn create_tender(env: Env, tender_id: u64, authority: Address, title: String) {
        let inst = env.storage().instance();
        let t_key = Self::tender_key(tender_id);

        if inst.has(&t_key) {
            panic!("tender_id exists");
        }

        let tender = Tender {
            tender_id,
            authority,
            title,
            status: TenderStatus::Open,
            winning_bid: None,
        };

        let b_key = Self::bids_key(tender_id);
        let empty: Vec<Bid> = Vec::new(&env);

        inst.set(&t_key, &tender);
        inst.set(&b_key, &empty);
    }

    // Vendor submits a bid for an open tender
    pub fn submit_bid(
        env: Env,
        bid_id: u64,
        tender_id: u64,
        vendor: Address,
        amount: i128,
        details: String,
    ) {
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let inst = env.storage().instance();
        let t_key = Self::tender_key(tender_id);
        let b_key = Self::bids_key(tender_id);

        let tender: Tender =
            inst.get(&t_key).unwrap_or_else(|| panic!("tender not found"));

        if let TenderStatus::Open = tender.status {
        } else {
            panic!("tender not open");
        }

        let mut bids: Vec<Bid> =
            inst.get(&b_key).unwrap_or_else(|| panic!("bids missing"));

        // ensure unique bid_id per tender
        for b in bids.iter() {
            if b.bid_id == bid_id {
                panic!("bid_id exists");
            }
        }

        let bid = Bid {
            bid_id,
            tender_id,
            vendor,
            amount,
            details,
        };
        bids.push_back(bid);

        inst.set(&b_key, &bids);
    }

    // Authority awards tender by selecting a winning bid
    pub fn award_tender(env: Env, tender_id: u64, caller: Address, winning_bid_id: u64) {
        let inst = env.storage().instance();
        let t_key = Self::tender_key(tender_id);
        let b_key = Self::bids_key(tender_id);

        let mut tender: Tender =
            inst.get(&t_key).unwrap_or_else(|| panic!("tender not found"));

        if caller != tender.authority {
            panic!("only authority can award");
        }

        if let TenderStatus::Open = tender.status {
        } else {
            panic!("tender not open");
        }

        let bids: Vec<Bid> =
            inst.get(&b_key).unwrap_or_else(|| panic!("bids missing"));

        let mut found = false;
        for b in bids.iter() {
            if b.bid_id == winning_bid_id {
                found = true;
                break;
            }
        }
        if !found {
            panic!("winning bid not found");
        }

        tender.status = TenderStatus::Awarded;
        tender.winning_bid = Some(winning_bid_id);

        inst.set(&t_key, &tender);
    }

    // Authority cancels an open tender
    pub fn cancel_tender(env: Env, tender_id: u64, caller: Address) {
        let inst = env.storage().instance();
        let t_key = Self::tender_key(tender_id);

        let mut tender: Tender =
            inst.get(&t_key).unwrap_or_else(|| panic!("tender not found"));

        if caller != tender.authority {
            panic!("only authority can cancel");
        }

        if let TenderStatus::Open = tender.status {
        } else {
            panic!("can cancel only when open");
        }

        tender.status = TenderStatus::Cancelled;
        inst.set(&t_key, &tender);
    }

    // Views
    pub fn get_tender(env: Env, tender_id: u64) -> Option<Tender> {
        let inst = env.storage().instance();
        let key = Self::tender_key(tender_id);
        inst.get(&key)
    }

    pub fn get_bids(env: Env, tender_id: u64) -> Option<Vec<Bid>> {
        let inst = env.storage().instance();
        let key = Self::bids_key(tender_id);
        inst.get(&key)
    }

    fn tender_key(id: u64) -> (Symbol, u64) {
        (TENDER_NS, id)
    }

    fn bids_key(id: u64) -> (Symbol, Symbol, u64) {
        (TENDER_NS, symbol_short!("BIDS"), id)
    }
}
