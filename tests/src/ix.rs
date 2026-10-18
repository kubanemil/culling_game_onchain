use anchor_lang::{
    prelude::Pubkey,
    solana_program::{bpf_loader_upgradeable, instruction::Instruction},
    system_program, InstructionData, ToAccountMetas,
};
use anchor_spl::{associated_token, token_2022::spl_token_2022};

use crate::harness::{ata, pda};

fn ix(program: Pubkey, data: impl InstructionData, accounts: impl ToAccountMetas) -> Instruction {
    Instruction::new_with_bytes(program, &data.data(), accounts.to_account_metas(None))
}

fn program_data(program: &Pubkey) -> Pubkey {
    pda(&[program.as_ref()], &bpf_loader_upgradeable::ID)
}

pub mod sale {
    use ::sale::{
        accounts as a, instruction as i, InitializeArgs, RoundArgs, ALLOCATION_SEED, CONFIG_SEED,
        ROUND_SEED, TOKEN_MINT,
    };

    use super::*;

    pub fn config() -> Pubkey {
        pda(&[CONFIG_SEED], &::sale::ID)
    }

    pub fn round(id: u64) -> Pubkey {
        pda(&[ROUND_SEED, &id.to_le_bytes()], &::sale::ID)
    }

    pub fn allocation(round: &Pubkey, buyer: &Pubkey) -> Pubkey {
        pda(
            &[ALLOCATION_SEED, round.as_ref(), buyer.as_ref()],
            &::sale::ID,
        )
    }

    pub fn initialize(admin: &Pubkey, args: InitializeArgs) -> Instruction {
        ix(
            ::sale::ID,
            i::Initialize { args },
            a::Initialize {
                admin: *admin,
                config: config(),
                mint: TOKEN_MINT,
                program_data: program_data(&::sale::ID),
                token_program: spl_token_2022::ID,
                system_program: system_program::ID,
            },
        )
    }

    pub fn propose_admin(admin: &Pubkey, new_admin: Pubkey) -> Instruction {
        ix(
            ::sale::ID,
            i::ProposeAdmin { new_admin },
            a::ProposeAdmin {
                admin: *admin,
                config: config(),
            },
        )
    }

    pub fn accept_admin(new_admin: &Pubkey) -> Instruction {
        ix(
            ::sale::ID,
            i::AcceptAdmin {},
            a::AcceptAdmin {
                new_admin: *new_admin,
                config: config(),
            },
        )
    }

    pub fn create_round(admin: &Pubkey, id: u64, args: RoundArgs) -> Instruction {
        ix(
            ::sale::ID,
            i::CreateRound { args },
            a::CreateRound {
                admin: *admin,
                config: config(),
                round: round(id),
                system_program: system_program::ID,
            },
        )
    }

    pub fn set_paused(admin: &Pubkey, id: u64, paused: bool) -> Instruction {
        ix(
            ::sale::ID,
            i::SetPaused { paused },
            a::SetPaused {
                admin: *admin,
                config: config(),
                round: round(id),
            },
        )
    }

    pub fn buy(
        buyer: &Pubkey,
        id: u64,
        amount: u64,
        max_amount: u64,
        proof: Vec<[u8; 32]>,
    ) -> Instruction {
        ix(
            ::sale::ID,
            i::Buy {
                amount,
                max_amount,
                proof,
            },
            a::Buy {
                buyer: *buyer,
                config: config(),
                round: round(id),
                allocation: allocation(&round(id), buyer),
                system_program: system_program::ID,
            },
        )
    }

    pub fn claim(buyer: &Pubkey, id: u64) -> Instruction {
        claim_with(buyer, id, allocation(&round(id), buyer))
    }

    pub fn claim_with(buyer: &Pubkey, id: u64, allocation: Pubkey) -> Instruction {
        ix(
            ::sale::ID,
            i::Claim {},
            a::Claim {
                buyer: *buyer,
                config: config(),
                mint: TOKEN_MINT,
                round: round(id),
                allocation,
                buyer_tokens: ata(buyer, &TOKEN_MINT),
                token_program: spl_token_2022::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
        )
    }

    pub fn finalize_round(id: u64) -> Instruction {
        ix(
            ::sale::ID,
            i::FinalizeRound {},
            a::FinalizeRound {
                config: config(),
                round: round(id),
            },
        )
    }

    pub fn withdraw_proceeds(admin: &Pubkey, id: u64) -> Instruction {
        ix(
            ::sale::ID,
            i::WithdrawProceeds {},
            a::WithdrawProceeds {
                admin: *admin,
                config: config(),
                round: round(id),
            },
        )
    }
}

pub mod asset {
    use ::asset::{
        accounts as a, instruction as i, CardField, CreateCardArgs, CARD_MINT_SEED, CARD_SEED,
        CONFIG_SEED,
    };

    use super::*;

    pub fn config() -> Pubkey {
        pda(&[CONFIG_SEED], &::asset::ID)
    }

    pub fn card_mint(id: u16) -> Pubkey {
        pda(&[CARD_MINT_SEED, &id.to_le_bytes()], &::asset::ID)
    }

    pub fn card(mint: &Pubkey) -> Pubkey {
        pda(&[CARD_SEED, mint.as_ref()], &::asset::ID)
    }

    pub fn initialize(admin: &Pubkey) -> Instruction {
        ix(
            ::asset::ID,
            i::Initialize {},
            a::Initialize {
                admin: *admin,
                config: config(),
                program_data: program_data(&::asset::ID),
                system_program: system_program::ID,
            },
        )
    }

    pub fn propose_admin(admin: &Pubkey, new_admin: Pubkey) -> Instruction {
        ix(
            ::asset::ID,
            i::ProposeAdmin { new_admin },
            a::ProposeAdmin {
                admin: *admin,
                config: config(),
            },
        )
    }

    pub fn accept_admin(new_admin: &Pubkey) -> Instruction {
        ix(
            ::asset::ID,
            i::AcceptAdmin {},
            a::AcceptAdmin {
                new_admin: *new_admin,
                config: config(),
            },
        )
    }

    pub fn create_card(admin: &Pubkey, id: u16, args: CreateCardArgs) -> Instruction {
        let mint = card_mint(id);
        ix(
            ::asset::ID,
            i::CreateCard { args },
            a::CreateCard {
                admin: *admin,
                config: config(),
                mint,
                card: card(&mint),
                token_program: spl_token_2022::ID,
                system_program: system_program::ID,
            },
        )
    }

    pub fn buy_card(buyer: &Pubkey, card_mint: &Pubkey) -> Instruction {
        buy_card_with(buyer, card_mint, &::sale::TOKEN_MINT)
    }

    pub fn buy_card_with(buyer: &Pubkey, card_mint: &Pubkey, token_mint: &Pubkey) -> Instruction {
        ix(
            ::asset::ID,
            i::BuyCard {},
            a::BuyCard {
                buyer: *buyer,
                config: config(),
                card: card(card_mint),
                card_mint: *card_mint,
                token_mint: *token_mint,
                buyer_tokens: ata(buyer, token_mint),
                buyer_cards: ata(buyer, card_mint),
                token_program: spl_token_2022::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
        )
    }

    pub fn update_card_metadata(
        authority: &Pubkey,
        payer: &Pubkey,
        card_mint: &Pubkey,
        field: CardField,
        value: &str,
    ) -> Instruction {
        ix(
            ::asset::ID,
            i::UpdateCardMetadata {
                field,
                value: value.to_string(),
            },
            a::UpdateCardMetadata {
                governance_authority: *authority,
                payer: *payer,
                config: config(),
                card: card(card_mint),
                card_mint: *card_mint,
                token_program: spl_token_2022::ID,
                system_program: system_program::ID,
            },
        )
    }
}

pub mod duel {
    use ::duel::{accounts as a, instruction as i, DUEL_SEED, ESCROW_SEED};

    use super::*;

    pub fn address(creator: &Pubkey, nonce: u64) -> Pubkey {
        pda(
            &[DUEL_SEED, creator.as_ref(), &nonce.to_le_bytes()],
            &::duel::ID,
        )
    }

    pub fn create(
        creator: &Pubkey,
        nonce: u64,
        stake: u64,
        commitment: [u8; 32],
        opponent: Option<Pubkey>,
    ) -> Instruction {
        ix(
            ::duel::ID,
            i::CreateDuel {
                nonce,
                stake,
                commitment,
                opponent,
            },
            a::CreateDuel {
                creator: *creator,
                duel: address(creator, nonce),
                system_program: system_program::ID,
            },
        )
    }

    pub fn accept(opponent: &Pubkey, duel: &Pubkey, commitment: [u8; 32]) -> Instruction {
        ix(
            ::duel::ID,
            i::AcceptDuel { commitment },
            a::AcceptDuel {
                opponent: *opponent,
                duel: *duel,
                system_program: system_program::ID,
            },
        )
    }

    pub fn cancel(creator: &Pubkey, duel: &Pubkey) -> Instruction {
        ix(
            ::duel::ID,
            i::CancelDuel {},
            a::CancelDuel {
                creator: *creator,
                duel: *duel,
            },
        )
    }

    pub fn escrow(duel: &Pubkey, player: &Pubkey) -> Pubkey {
        pda(&[ESCROW_SEED, duel.as_ref(), player.as_ref()], &::duel::ID)
    }

    pub fn reveal(
        player: &Pubkey,
        duel: &Pubkey,
        card_mint: &Pubkey,
        secret: [u8; 32],
    ) -> Instruction {
        ix(
            ::duel::ID,
            i::Reveal { secret },
            a::Reveal {
                player: *player,
                duel: *duel,
                card: super::asset::card(card_mint),
                card_mint: *card_mint,
                player_cards: ata(player, card_mint),
                escrow: escrow(duel, player),
                token_program: spl_token_2022::ID,
                system_program: system_program::ID,
            },
        )
    }

    /// `revealed` is the card the player revealed, if any; it comes back with the claim.
    pub fn claim(player: &Pubkey, duel: &Pubkey, revealed: Option<&Pubkey>) -> Instruction {
        ix(
            ::duel::ID,
            i::Claim {},
            a::Claim {
                player: *player,
                duel: *duel,
                card_mint: revealed.copied(),
                escrow: revealed.map(|_| escrow(duel, player)),
                player_cards: revealed.map(|mint| ata(player, mint)),
                token_program: spl_token_2022::ID,
            },
        )
    }
}

pub mod governance {
    use ::asset::CardField;
    use ::governance::{
        accounts as a, instruction as i, InitializeArgs, AUTHORITY, CONFIG_SEED, FIELD_SEED,
        PROPOSAL_SEED, VOTER_SEED, VOTE_SEED,
    };
    use ::sale::TOKEN_MINT;

    use super::*;

    pub fn config() -> Pubkey {
        pda(&[CONFIG_SEED], &::governance::ID)
    }

    pub fn proposal(id: u64) -> Pubkey {
        pda(&[PROPOSAL_SEED, &id.to_le_bytes()], &::governance::ID)
    }

    pub fn voter(owner: &Pubkey) -> Pubkey {
        pda(&[VOTER_SEED, owner.as_ref()], &::governance::ID)
    }

    pub fn vote_record(proposal: &Pubkey, owner: &Pubkey) -> Pubkey {
        pda(
            &[VOTE_SEED, proposal.as_ref(), owner.as_ref()],
            &::governance::ID,
        )
    }

    pub fn escrow(owner: &Pubkey) -> Pubkey {
        ata(&voter(owner), &TOKEN_MINT)
    }

    pub fn initialize(admin: &Pubkey, args: InitializeArgs) -> Instruction {
        ix(
            ::governance::ID,
            i::Initialize { args },
            a::Initialize {
                admin: *admin,
                config: config(),
                mint: TOKEN_MINT,
                program_data: program_data(&::governance::ID),
                system_program: system_program::ID,
            },
        )
    }

    pub fn create_proposal(
        proposer: &Pubkey,
        id: u64,
        card_mint: &Pubkey,
        field: CardField,
        value: &str,
    ) -> Instruction {
        ix(
            ::governance::ID,
            i::CreateProposal {
                field,
                value: value.to_string(),
            },
            a::CreateProposal {
                proposer: *proposer,
                config: config(),
                card: super::asset::card(card_mint),
                proposal: proposal(id),
                system_program: system_program::ID,
            },
        )
    }

    pub fn deposit(owner: &Pubkey, amount: u64) -> Instruction {
        deposit_with(owner, amount, &TOKEN_MINT)
    }

    pub fn deposit_with(owner: &Pubkey, amount: u64, mint: &Pubkey) -> Instruction {
        ix(
            ::governance::ID,
            i::Deposit { amount },
            a::Deposit {
                owner: *owner,
                config: config(),
                mint: *mint,
                voter: voter(owner),
                escrow: ata(&voter(owner), mint),
                owner_tokens: ata(owner, mint),
                token_program: spl_token_2022::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
        )
    }

    pub fn withdraw(owner: &Pubkey, amount: u64) -> Instruction {
        ix(
            ::governance::ID,
            i::Withdraw { amount },
            a::Withdraw {
                owner: *owner,
                config: config(),
                mint: TOKEN_MINT,
                voter: voter(owner),
                escrow: escrow(owner),
                owner_tokens: ata(owner, &TOKEN_MINT),
                token_program: spl_token_2022::ID,
            },
        )
    }

    pub fn vote(owner: &Pubkey, id: u64, approve: bool) -> Instruction {
        vote_with(owner, id, approve, &TOKEN_MINT)
    }

    pub fn vote_with(owner: &Pubkey, id: u64, approve: bool, mint: &Pubkey) -> Instruction {
        ix(
            ::governance::ID,
            i::Vote { approve },
            a::CastVote {
                owner: *owner,
                config: config(),
                mint: *mint,
                proposal: proposal(id),
                voter: voter(owner),
                escrow: ata(&voter(owner), mint),
                vote: vote_record(&proposal(id), owner),
                token_program: spl_token_2022::ID,
                system_program: system_program::ID,
            },
        )
    }

    pub fn close_vote(owner: &Pubkey, id: u64) -> Instruction {
        ix(
            ::governance::ID,
            i::CloseVote {},
            a::CloseVote {
                owner: *owner,
                proposal: proposal(id),
                vote: vote_record(&proposal(id), owner),
            },
        )
    }

    pub fn field_version(card_mint: &Pubkey, field: CardField) -> Pubkey {
        pda(
            &[FIELD_SEED, card_mint.as_ref(), &[field as u8]],
            &::governance::ID,
        )
    }

    pub fn execute(payer: &Pubkey, id: u64, card_mint: &Pubkey, field: CardField) -> Instruction {
        ix(
            ::governance::ID,
            i::ExecuteProposal {},
            a::ExecuteProposal {
                payer: *payer,
                config: config(),
                proposal: proposal(id),
                field_version: field_version(card_mint, field),
                authority: AUTHORITY,
                asset_config: super::asset::config(),
                card: super::asset::card(card_mint),
                card_mint: *card_mint,
                asset_program: ::asset::ID,
                token_program: spl_token_2022::ID,
                system_program: system_program::ID,
            },
        )
    }
}
