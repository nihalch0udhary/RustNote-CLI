// contracts/rustnote/src/lib.rs
#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String, Symbol, symbol_short,
};

#[contracttype]
#[derive(Clone)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub ts: u64,
}

// Storage key enums
#[contracttype]
pub enum NotesKey {
    Count(Address),               // u64 counter per user
    Record(Address, u64),         // Note record stored by (owner, id)
}

const NOTES_COUNT_PREFIX: Symbol = symbol_short!("N_COUNT");

#[contract]
pub struct RustNote;

#[contractimpl]
impl RustNote {
    // Add a note for the caller. Caller must supply their Address and be authorized.
    // Returns the new note id (u64).
    pub fn add_note(env: Env, caller: Address, title: String, body: String) -> u64 {
        // require the caller to have authorized this invocation
        caller.require_auth();  // Fixed: removed &env argument

        // get current count
        let key_count = NotesKey::Count(caller.clone());
        let mut count: u64 = env.storage().instance().get(&key_count).unwrap_or(0u64);
        count = count.saturating_add(1);
        env.storage().instance().set(&key_count, &count);

        // timestamp
        let ts = env.ledger().timestamp();

        let note = Note {
            id: count,
            title,
            body,
            ts,
        };

        // store note under (caller, id)
        env.storage().instance().set(&NotesKey::Record(caller.clone(), count), &note);

        // Extend TTL for persistence
        env.storage().instance().extend_ttl(5000, 5000);

        count
    }

    // View a note for a given owner and id.
    // Anyone can read; to modify/remove they must pass owner address and be auth'd.
    pub fn view_note(env: Env, owner: Address, id: u64) -> Note {
        env.storage()
            .instance()
            .get(&NotesKey::Record(owner, id))
            .expect("note not found")
    }

    // Return the number of notes a user has (useful for listing client-side)
    pub fn notes_count(env: Env, owner: Address) -> u64 {
        env.storage().instance().get(&NotesKey::Count(owner)).unwrap_or(0u64)
    }

    // Remove a note: caller passes owner (must be same as caller and authorized) and id.
    pub fn remove_note(env: Env, caller: Address, id: u64) {
        caller.require_auth();  // Fixed: removed &env argument

        // confirm note exists and owner is caller
        let key = NotesKey::Record(caller.clone(), id);
        let _note: Note = env.storage().instance().get(&key).expect("note not found");

        // delete by setting to default? there's no direct delete helper, set to zeroed/default marker.
        // To keep it simple, remove by setting a "DELETED" marker
        let deleted = Note {
            id,
            title: String::from_str(&env, "DELETED"),
            body: String::from_str(&env, ""),
            ts: env.ledger().timestamp(),
        };
        env.storage().instance().set(&key, &deleted);

        // Extend TTL for persistence
        env.storage().instance().extend_ttl(5000, 5000);
    }
}