use anchor_lang::prelude::*;


#[derive(Debug, Clone, Copy)]
pub enum Grade {
    Third,
    Second,
    First,
    Special,
}


#[account]
pub struct Card {
    hp: u8,
    grade: u8,
    property: String
}


impl Card {
    const INIT_SPACE: usize = 8 + 1 + 1 + 64;
}
