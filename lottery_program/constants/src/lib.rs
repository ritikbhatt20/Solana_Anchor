use anchor_lang::{
    prelude::*,
    solana_program::{clock::Clock, hash::hash, program::invoke, system_instruction::transfer},
};

mod constants;
mod error;

use crate::{constants::*, error::*};

declare_id!("9fKCq3AKFsWvAoSreM1VC8ULq9sbf4wPjUZvHBPzoN6L");

#[program]
mod lottery {
    use super::*;

    pub fn init_master(_ctx: Context<InitMaster>) -> Result<()> {
        //Write the logic in here
        //What is master -> an object that holds last lottery id
        Ok(())
    }

    // Create a lottery
    pub fn create_lottery(ctx: Context<CreateLottery>, ticket_price: u64) -> Result<()> {
        // Create a lottery account
        // What is a Lottery account? -> it holds the id, the winning address, each ticket cost price, if the prize was claimed, who has authority over the lottery

        let lottery = &mut ctx.accounts.lottery;
        let master = &mut ctx.accounts.master;

        // Increment the last lottery id
        master.last_id += 1;

        // Set the lottery values
        lottery.id = master.last_id;
        lottery.authority = ctx.accounts.authority.key();
        lottery.ticket_price = ticket_price;

        msg!("Created Lottery: {}", lottery.id);
        msg!("Authority: {}", lottery.authority);
        msg!("Ticket Price: {}", lottery.ticket_price);

        Ok(())
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>, lottery_id: u32) -> Result<()> {
        // When we buy a ticket, we create a ticket account and transfer the SOL from buyer to lottery account
        let lottery = &mut ctx.accounts.lottery;
        let ticket = &mut ctx.accounts.ticket;
        let buyer = &ctx.accounts.buyer;

        if lottery.winner_id.is_some() {
            return err!(LotteryError::WinnerAlreadyExists);
        }

        invoke(
            &transfer(&buyer.key(), &lottery.key(), lottery.ticket_price),
            &[
                buyer.to_account_info(),
                lottery.to_account_info(),
                ctx.accounts.system_program.to_account_info()
            ],
        )?;

        lottery.last_ticket_id += 1;        

        ticket.id = lottery.last_ticket_id;
        ticket.lottery_id = lottery_id;
        ticket.authority = buyer.key();

        msg!("Ticket id: {}", ticket.id);
        msg!("Ticket authority: {}", ticket.authority);

        Ok(())                              
    }

    pub fn pick_winner(ctx: Context<PickWinner>, lottery_id: u32) -> Result<()> {
        // select a random ticket as a winner and set the winner_id to that winner

        let lottery = &mut ctx.accounts.lottery;

        // Pick a pseudo random winner
        let clock = Clock::get()?;
        let pseudo_random_number = ((u64::from_le_bytes(
            <[u8;8]>::try_from(&hash(&clock.unix_timestamp.to_be_bytes()).to_bytes()[..8]).unwrap(),
        ) * clock.slot)
            % u32::MAX as u64) as u32;

        let winner_id = (pseudo_random_number % lottery.last_ticket_id) + 1;

        lottery.winner_id = Some(winner_id);

        msg!("Winner id:{}", winner_id);
        Ok(())
    }
}                                           

// What are Accounts -> We use program to generate Accounts and each of these accounts have a Public Key and they hold data
// Seeds -> We can predictably find an address or generate a public key of an account 
#[derive(Accounts)]
pub struct InitMaster<'info> {
    #[account(
        init,
        payer = payer,
        space = 4 + 8,
        seeds = [MASTER_SEED.as_bytes()],
        bump,
    )]
    pub master: Account<'info, Master>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Master {
    pub last_id: u32, // gonna take 4 space bytes
}

#[derive(Accounts)]
pub struct CreateLottery<'info> {
    #[account(
        init,
        payer = authority,
        space = 4 + 32 + 8 + 4 + 1 + 4 + 1 + 8,  // last 8 is filler space(count discriminator)
        seeds = [LOTTERY_SEED.as_bytes(), &(master.last_id + 1).to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,

    // we also need access to the master account
    #[account(
        mut,
        seeds = [MASTER_SEED.as_bytes()],
        bump,
    )]
    pub master: Account<'info, Master>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

}

#[account]
pub struct Lottery {
    pub id: u32,
    pub authority: Pubkey,
    pub ticket_price: u64,
    pub last_ticket_id: u32,
    pub winner_id: Option<u32>,
    pub claimed: bool,
}

#[derive(Accounts)]
#[instruction(lottery_id: u32)]
pub struct BuyTicket<'info> {
    // we need to acccess to lottery account to find which lottery we are buying ticket for
    #[account(
        mut,
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()],
        bump
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(
        init,
        payer = buyer,
        space = 4 + 4 + 32 + 8,
        seeds = [
            TICKET_SEED.as_bytes(),
            lottery.key().as_ref(),
            &(lottery.last_ticket_id + 1).to_le_bytes(),
        ],
        bump
    )]
    pub ticket: Account<'info, Ticket>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Ticket {
    pub id : u32,
    pub authority: Pubkey,
    pub lottery_id: u32,
}

#[derive(Accounts)]
#[instruction(lottery_id: u32)]
pub struct PickWinner<'info> {
    #[account(
        mut,
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()],
        bump,
        has_one = authority,
    )]
    pub lottery: Account<'info, Lottery>,

    pub authority: Signer<'info>,
}
