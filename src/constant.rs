pub const hbar:f64=1.0545718e-34;
pub const timestep:f64=5e-6;
pub const g_c:f64 = 9.80665;
pub const exp:f64 = 2.718281828;
pub const pi:f64 = 3.1415926535;

//pub const bohr_mag :f64 = 0.0;
pub const bohr_mag :f64 = 9.274e-24;
pub const boltz_const:f64 = 1.38e-23;
pub const MRb:f64 = 1.4192261e-25;
//100.0 is temp for convienience
pub const trans_width:f64 = pi* 64e6;
pub const c:f64 = 2.998e8;
pub const atom_frequency:f64 = c / 461e-9+ 2./pi*trans_width;
pub const force_multiplier :f64 = 1.0;

pub const mup:f64 = -bohr_mag;
pub const mum:f64 = bohr_mag;
pub const muz:f64 = 0.;
pub const sat_inten :f64 = 35.4;