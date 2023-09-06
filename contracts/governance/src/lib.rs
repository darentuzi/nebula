//! Governance contract
//!
//! This contract provides the implementation of
//! a stake controlled DAO that allows participants
//! vote on code upgrades.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, Env,
    Map, Symbol,
};

mod voting_contract {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/voting.wasm");
}

/// Datakey holds all possible storage keys this
/// contract uses. See https://soroban.stellar.org/docs/getting-started/storing-data .
#[derive(Clone, Copy)]
#[contracttype]
enum DataKey {
    Initialized = 1,
    Curator = 2,
    Token = 3,
    Participants = 4,
    VotingContractAddress = 5,
}

/// All the expected errors this contract expects.
/// This error codes will appear as output in the transaction
/// receipt.
#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The contract should be only initialized once.
    AlreadyInitialized = 1,
    // Must have funds for the operation.
    InsufficientFunds = 2,
    // Certain amounts are not valid in some operations.(Like under and/or equal to zero)
    InvalidAmount = 3,
    ParticipantNotFound = 4,
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn init(env: Env, curator: Address, token: Address, voting_contract: Address) {
        let storage = env.storage().persistent();

        if storage.has(&DataKey::Initialized) {
            panic_with_error!(&env, Error::AlreadyInitialized)
        }

        storage.set(&DataKey::Initialized, &());
        storage.set(&DataKey::Curator, &curator);
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::VotingContractAddress, &voting_contract);
        storage.set(
            &DataKey::Participants,
            &Map::<Address, Participant>::new(&env),
        );
    }

    pub fn join(env: Env, participant_addr: Address, amount: i128) -> Result<(), Error> {
        participant_addr.require_auth();

        let storage = env.storage().persistent();

        let mut participants = storage
            .get::<_, Map<Address, Participant>>(&DataKey::Participants)
            .unwrap();

        let mut participant = Participant::new(participant_addr.clone());

        Self::stake_funds(&env, &mut participant, amount)?;

        participants.set(participant_addr.clone(), participant);

        storage.set(&DataKey::Participants, &participants);

        env.events().publish(
            (Symbol::new(&env, "participant_joined"), participant_addr),
            (),
        );
        Ok(())
    }

    fn stake_funds(env: &Env, participant: &mut Participant, amount: i128) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let storage = env.storage().persistent();
        let token_addr = storage.get::<_, Address>(&DataKey::Token).unwrap();
        let token_client = token::Client::new(env, &token_addr);
        let balance = token_client.balance(&participant.address);

        if balance < amount {
            return Err(Error::InsufficientFunds);
        }

        token_client.transfer(
            &participant.address,
            &env.current_contract_address(),
            &amount,
        );

        participant.current_balance += amount;

        env.events()
            .publish((Symbol::new(env, "stake"), &participant.address), amount);
        Ok(())
    }

    pub fn stake(env: Env, participant: Address, amount: i128) -> Result<(), Error> {
        participant.require_auth();

        let storage = env.storage().persistent();

        let mut participants = storage
            .get::<_, Map<Address, Participant>>(&DataKey::Participants)
            .unwrap();

        let mut stored_participant = participants
            .get(participant.clone())
            .ok_or(Error::ParticipantNotFound)?;

        Self::stake_funds(&env, &mut stored_participant, amount)?;

        participants.set(participant.clone(), stored_participant);
        storage.set(&DataKey::Participants, &participants);

        Ok(())
    }

    pub fn leave(env: Env, participant: Address) -> Result<(), Error> {
        participant.require_auth();

        let storage = env.storage().persistent();

        let mut participants = storage
            .get::<_, Map<Address, Participant>>(&DataKey::Participants)
            .unwrap();

        let mut stored_participant = participants
            .get(participant.clone())
            .ok_or(Error::ParticipantNotFound)?;

        let amount = stored_participant.current_balance;

        Self::withdraw_funds(&env, &mut stored_participant, amount).unwrap();

        participants.remove(participant.clone());
        storage.set(&DataKey::Participants, &participants);

        env.events()
            .publish((Symbol::new(&env, "participant_left"), &participant), ());

        Ok(())
    }

    fn withdraw_funds(env: &Env, participant: &mut Participant, amount: i128) -> Result<(), Error> {
        if participant.current_balance < amount {
            return Err(Error::InsufficientFunds);
        }

        let storage = env.storage().persistent();
        let token_addr = storage.get::<_, Address>(&DataKey::Token).unwrap();
        let token_client = token::Client::new(env, &token_addr);

        token_client.transfer(
            &env.current_contract_address(),
            &participant.address,
            &amount,
        );

        participant.current_balance -= amount;

        env.events()
            .publish((Symbol::new(env, "withdraw"), &participant.address), amount);

        Ok(())
    }

    pub fn withdraw(env: Env, participant: Address, amount: i128) -> Result<(), Error> {
        participant.require_auth();

        let storage = env.storage().persistent();

        let mut participants = storage
            .get::<_, Map<Address, Participant>>(&DataKey::Participants)
            .unwrap();

        let mut stored_participant = participants
            .get(participant.clone())
            .ok_or(Error::ParticipantNotFound)?;

        Self::withdraw_funds(&env, &mut stored_participant, amount)?;

        participants.set(participant.clone(), stored_participant);
        storage.set(&DataKey::Participants, &participants);
        Ok(())
    }
}

#[contracttype]
struct Participant {
    address: Address,
    whitelisted: bool,
    current_balance: i128,
}

impl Participant {
    pub fn new(address: Address) -> Self {
        Participant {
            address,
            whitelisted: false,
            current_balance: 0,
        }
    }
}

#[cfg(test)]
mod test;
