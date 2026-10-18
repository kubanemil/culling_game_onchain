use std::{fmt::Debug, path::PathBuf};

use anchor_lang::{
    prelude::{Clock, Pubkey},
    solana_program::{
        bpf_loader_upgradeable, instruction::Instruction, program_pack::Pack, system_instruction,
    },
    AccountDeserialize,
};
use anchor_spl::{
    associated_token::{
        get_associated_token_address_with_program_id,
        spl_associated_token_account::instruction::create_associated_token_account_idempotent,
    },
    token_2022::spl_token_2022::{
        self,
        extension::{BaseStateWithExtensions, StateWithExtensions},
        state::Mint as MintState,
    },
    token_interface::{spl_token_metadata_interface::state::TokenMetadata, TokenAccount},
};
use litesvm::{
    types::{FailedTransactionMetadata, TransactionMetadata},
    LiteSVM,
};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub const SOL: u64 = 1_000_000_000;
pub const START_TIME: i64 = 1_800_000_000;

pub type TxResult = Result<TransactionMetadata, Box<FailedTransactionMetadata>>;

pub struct Env {
    pub svm: LiteSVM,
    pub deployer: Keypair,
}

impl Env {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        let deployer = Keypair::new();
        svm.airdrop(&deployer.pubkey(), 1_000 * SOL).unwrap();
        for (id, name) in [
            (sale::ID, "sale"),
            (asset::ID, "asset"),
            (duel::ID, "duel"),
            (governance::ID, "governance"),
        ] {
            svm.add_program_from_file(id, program_path(name))
                .unwrap_or_else(|e| panic!("load {name}.so (run `anchor build` first): {e}"));
            set_upgrade_authority(&mut svm, &id, &deployer.pubkey());
        }
        let mut clock = svm.get_sysvar::<Clock>();
        clock.unix_timestamp = START_TIME;
        svm.set_sysvar(&clock);
        Self { svm, deployer }
    }

    pub fn user(&mut self) -> Keypair {
        let user = Keypair::new();
        self.svm.airdrop(&user.pubkey(), 100 * SOL).unwrap();
        user
    }

    /// The first signer pays. A fresh blockhash per call keeps identical retries distinct.
    pub fn send(&mut self, ixs: &[Instruction], signers: &[&Keypair]) -> TxResult {
        self.svm.expire_blockhash();
        let message = Message::new_with_blockhash(
            ixs,
            Some(&signers[0].pubkey()),
            &self.svm.latest_blockhash(),
        );
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(message), signers)
            .expect("sign transaction");
        self.svm.send_transaction(tx).map_err(Box::new)
    }

    #[track_caller]
    pub fn ok(&mut self, ixs: &[Instruction], signers: &[&Keypair]) -> TransactionMetadata {
        match self.send(ixs, signers) {
            Ok(meta) => meta,
            Err(failed) => panic!(
                "transaction failed: {:?}\n{}",
                failed.err,
                failed.meta.logs.join("\n")
            ),
        }
    }

    pub fn now(&self) -> i64 {
        self.svm.get_sysvar::<Clock>().unix_timestamp
    }

    pub fn warp(&mut self, seconds: i64) {
        let mut clock = self.svm.get_sysvar::<Clock>();
        clock.unix_timestamp += seconds;
        clock.slot += 1;
        self.svm.set_sysvar(&clock);
    }

    pub fn warp_to(&mut self, timestamp: i64) {
        let now = self.now();
        self.warp(timestamp - now);
    }

    pub fn exists(&self, address: &Pubkey) -> bool {
        self.svm
            .get_account(address)
            .is_some_and(|account| account.lamports > 0)
    }

    pub fn lamports(&self, address: &Pubkey) -> u64 {
        self.svm.get_account(address).map_or(0, |a| a.lamports)
    }

    #[track_caller]
    pub fn account<T: AccountDeserialize>(&self, address: &Pubkey) -> T {
        let account = self
            .svm
            .get_account(address)
            .unwrap_or_else(|| panic!("account {address} does not exist"));
        T::try_deserialize(&mut account.data.as_slice()).expect("deserialize account")
    }

    pub fn token_balance(&self, address: &Pubkey) -> u64 {
        self.svm.get_account(address).map_or(0, |account| {
            TokenAccount::try_deserialize(&mut account.data.as_slice())
                .expect("token account")
                .amount
        })
    }

    pub fn mint_supply(&self, mint: &Pubkey) -> u64 {
        let account = self.svm.get_account(mint).expect("mint");
        StateWithExtensions::<MintState>::unpack(&account.data)
            .expect("mint")
            .base
            .supply
    }

    pub fn metadata(&self, mint: &Pubkey) -> TokenMetadata {
        let account = self.svm.get_account(mint).expect("mint");
        StateWithExtensions::<MintState>::unpack(&account.data)
            .expect("mint")
            .get_variable_len_extension::<TokenMetadata>()
            .expect("token metadata")
    }

    /// A Token-2022 mint the caller fully controls, for impersonation attacks.
    pub fn fake_mint(&mut self, authority: &Keypair, decimals: u8) -> Pubkey {
        let mint = Keypair::new();
        let rent = self.svm.minimum_balance_for_rent_exemption(MintState::LEN);
        let ixs = [
            system_instruction::create_account(
                &authority.pubkey(),
                &mint.pubkey(),
                rent,
                MintState::LEN as u64,
                &spl_token_2022::ID,
            ),
            spl_token_2022::instruction::initialize_mint2(
                &spl_token_2022::ID,
                &mint.pubkey(),
                &authority.pubkey(),
                None,
                decimals,
            )
            .unwrap(),
        ];
        self.ok(&ixs, &[authority, &mint]);
        mint.pubkey()
    }

    pub fn mint_to(&mut self, authority: &Keypair, mint: &Pubkey, owner: &Pubkey, amount: u64) {
        let ixs = [
            create_ata_ix(&authority.pubkey(), owner, mint),
            spl_token_2022::instruction::mint_to(
                &spl_token_2022::ID,
                mint,
                &ata(owner, mint),
                &authority.pubkey(),
                &[],
                amount,
            )
            .unwrap(),
        ];
        self.ok(&ixs, &[authority]);
    }

    pub fn transfer_tokens(
        &mut self,
        owner: &Keypair,
        mint: &Pubkey,
        to: &Pubkey,
        amount: u64,
        decimals: u8,
    ) {
        let ixs = [
            create_ata_ix(&owner.pubkey(), to, mint),
            spl_token_2022::instruction::transfer_checked(
                &spl_token_2022::ID,
                &ata(&owner.pubkey(), mint),
                mint,
                &ata(to, mint),
                &owner.pubkey(),
                &[],
                amount,
                decimals,
            )
            .unwrap(),
        ];
        self.ok(&ixs, &[owner]);
    }
}

pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, &spl_token_2022::ID)
}

pub fn create_ata_ix(payer: &Pubkey, owner: &Pubkey, mint: &Pubkey) -> Instruction {
    create_associated_token_account_idempotent(payer, owner, mint, &spl_token_2022::ID)
}

pub fn pda(seeds: &[&[u8]], program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, program).0
}

#[track_caller]
pub fn expect_error(result: TxResult, code: impl Debug) {
    expect_log(result, &format!("Error Code: {code:?}."));
}

#[track_caller]
pub fn expect_log(result: TxResult, needle: &str) {
    match result {
        Ok(meta) => panic!(
            "expected failure with {needle:?}, but it succeeded\n{}",
            meta.logs.join("\n")
        ),
        Err(failed) => assert!(
            failed.meta.logs.iter().any(|line| line.contains(needle)),
            "expected {needle:?} in logs, got {:?}\n{}",
            failed.err,
            failed.meta.logs.join("\n")
        ),
    }
}

fn program_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/deploy")
        .join(format!("{name}.so"))
}

/// LiteSVM deploys with no upgrade authority; set one so the `initialize` checks can run.
fn set_upgrade_authority(svm: &mut LiteSVM, program: &Pubkey, authority: &Pubkey) {
    let programdata = pda(&[program.as_ref()], &bpf_loader_upgradeable::ID);
    let mut account = svm.get_account(&programdata).expect("programdata");
    // Layout: u32 variant, u64 slot, Option<Pubkey>.
    account.data[12] = 1;
    account.data[13..45].copy_from_slice(authority.as_ref());
    svm.set_account(programdata, account).unwrap();
}
