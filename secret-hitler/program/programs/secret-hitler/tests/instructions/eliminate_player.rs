use anchor_lang::prelude::{Pubkey, ToAccountMetas};
use secret_hitler::ID_CONST as PROGRAM_ID;
use solana_sdk::instruction::Instruction;

#[allow(dead_code)]
pub fn eliminate_player(
    player: &Pubkey,
    game_data: &Pubkey,
    nomination: Option<Pubkey>,
) -> Instruction {
    Instruction {
        program_id: PROGRAM_ID,
        accounts: ToAccountMetas::to_account_metas(
            &secret_hitler::accounts::EliminatePlayer {
                player: *player,
                nomination,
                game_data: *game_data,
            },
            None,
        ),
        data: anchor_lang::InstructionData::data(
            &secret_hitler::instruction::EliminateInactivePlayer {},
        ),
    }
}
