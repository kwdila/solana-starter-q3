use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::state::{GameData, PlayerData};
use crate::{GameErrorCode, GameState};

#[derive(Accounts)]
pub struct LeaveGame<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        mut,
        close=player,
        seeds = [
            game_data.key().to_bytes().as_ref(),
            player.key().to_bytes().as_ref()
        ],
        bump = player_data.bump,
    )]
    pub player_data: Account<'info, PlayerData>,
    #[account(
        seeds= [
            b"deposit",
            game_data.key().to_bytes().as_ref()
        ],
        bump = game_data.deposit_vault_bump.unwrap(),
    )]
    pub deposit_vault: Option<SystemAccount<'info>>,
    #[account(
        seeds= [
        b"deposit_vault",
        game_data.key().to_bytes().as_ref()
    ],
        bump = game_data.bet_vault_bump.unwrap(),
    )]
    pub bet_vault: Option<SystemAccount<'info>>,
    #[account(
        mut,
        seeds = [
            b"secret_hitler",
            game_data.host.to_bytes().as_ref(),
        ],
        bump = game_data.bump,
        // constraint = game_data.host.ne(player.key) @GameErrorCode::HostPlayerLeaving,
        constraint = game_data.active_players.len() > 1 @GameErrorCode::LastPlayerLeaving, // You have to close game if you are the last person (host)
        constraint = game_data.game_state == GameState::Setup @GameErrorCode::InvalidGameState,
        constraint = game_data.active_players.contains(player.key) @GameErrorCode::PlayerNotInGame,
        constraint = game_data.entry_deposit.is_some() == deposit_vault.is_some() @GameErrorCode::DepositNotFound,
        constraint = game_data.bet_amount.is_some() == bet_vault.is_some() @GameErrorCode::BetNotFound,
    )]
    pub game_data: Account<'info, GameData>,
    pub system_program: Program<'info, System>,
}

impl<'info> LeaveGame<'info> {
    pub fn remove_player(&mut self) -> Result<()> {
        match self.game_data.entry_deposit {
            Some(amount) => {
                let accounts = Transfer {
                    to: self.player.to_account_info(),
                    from: self
                        .deposit_vault
                        .as_ref()
                        .ok_or(GameErrorCode::DepositNotFound)?
                        .to_account_info(), //this is checked in game_data account constraints
                };

                let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
                transfer(ctx, amount)?
            }
            None => (),
        }

        match self.game_data.bet_amount {
            Some(amount) => {
                let accounts = Transfer {
                    to: self.player.to_account_info(),
                    from: self
                        .bet_vault
                        .as_ref()
                        .ok_or(GameErrorCode::BetNotFound)?
                        .to_account_info(), //this is checked in game_data account constraints
                };

                let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
                transfer(ctx, amount)?
            }
            None => (),
        }

        let index = self
            .game_data
            .active_players
            .iter()
            .position(|player| player == self.player.key)
            .ok_or(GameErrorCode::PlayerNotInGame)?; // this is checked in the game_data account constraints

        self.game_data.active_players.swap_remove(index);

        // handle host player leaving
        if self.player.key() == self.game_data.host {
            self.game_data.host = self.game_data.active_players[0];
        }

        Ok(())
    }
}
