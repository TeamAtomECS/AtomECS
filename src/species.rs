//! Predefined species that can be used in AtomECS.
//! 
use crate::constant::BOHRMAG;
use crate::{transition, species};

transition!(Strontium88_461, 650_759_219_088_937.0, 32e6, 430.0, BOHRMAG, -BOHRMAG, 0.0);
species!(Strontium88, Strontium88_461, 88);

transition!(Rubidium87_780D2, 384_228_115_202_521.0, 6.065e6, 16.69, BOHRMAG, -BOHRMAG, 0.0); //[Steck, 87 D2]
species!(Rubidium87, Rubidium87_780D2, 87);