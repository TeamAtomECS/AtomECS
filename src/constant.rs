// Reduced plank constant in SI units
pub const HBAR:f64=1.0545718e-34;

// Gravitational acceleration in SI units
#[allow(dead_code)]
pub const GC:f64 = 9.80665;

// Mathematical constant exp(1)
pub const EXP:f64 = 2.718281828;

// Mathematica constant pi
pub const PI:f64 = 3.1415926535;

//pub const bohr_mag :f64 = 0.0;
pub const BOHRMAG :f64 = 9.274e-24;

// Boltzmann constant in SI units
pub const BOLTZCONST:f64 = 1.38e-23;

pub const MRB:f64 = 1.4192261e-25;

//100.0 is temp for convienience
pub const TRANSWIDTH:f64 = PI* 64e6;

// Speed of light in SI units of m/s
pub const C:f64 = 2.998e8;


pub const ATOMFREQUENCY:f64 = C / 461e-9+ 2./PI*TRANSWIDTH;


pub const MUP:f64 = -BOHRMAG;
pub const MUM:f64 = BOHRMAG;
pub const MUZ:f64 = 0.;
pub const SATINTEN :f64 = 35.4;