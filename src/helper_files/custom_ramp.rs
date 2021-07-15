use crate::laser::gaussian::GaussianBeam;
use crate::ramp::Ramp;

pub fn get_gaussian_eth_ramp(
    duration: f64,
    steps: i64,
    distance: f64,
    gaussian: GaussianBeam,
) -> Ramp<GaussianBeam> {
    let mut frames = Vec::new();
    for i in 0..steps {
        let t = i as f64 * duration / steps as f64;
        frames.push((
            t,
            GaussianBeam {
                intersection: gaussian.direction * focus_eth_ramp(t, &duration, &distance),
                e_radius: gaussian.e_radius,
                power: gaussian.power,
                direction: gaussian.direction,
                rayleigh_range: gaussian.rayleigh_range,
                ellipticity: gaussian.ellipticity,
            },
        ));
    }
    let ramp = Ramp::new(frames);
    return ramp;
}

pub fn focus_eth_ramp(t: f64, duration: &f64, distance: &f64) -> f64 {
    let a0 = 6.0 * distance / (duration.powf(2.0));
    let focus = 1. / 2. * a0 * t.powf(2.0) - 1. / (3. * duration) * a0 * t.powf(3.0);
    println!("focus is: {}", focus);
    return focus;
}
