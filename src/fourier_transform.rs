use num::complex::{Complex32, ComplexFloat};
use std::f32::consts::PI;

/** Descrete Fourier Transform O(n^2)*/
pub fn dft(samples: &[f32]) -> Vec<f32> {
    let n = samples.len();
    let mut out: Vec<f32> = vec![0.0; n];
    for f in 0..n {
        let mut c: Complex32 = Complex32::new(0.0, 0.0);
        for i in 0..n {
            /* create a wave of f frequency, multiple it with the samples wave,
             * sum of multiplications will tell if the frequency is present or not.
             * positive vaule means the frequency is present.*/
            let t = i as f32 / n as f32;
            let theta = 2.0 * PI * f as f32 * t;
            let e = Complex32::new(0.0, theta).exp();
            c += samples[i] * e
        }
        // println!("cos:{:.2} sin:{:.2}", c.re(), c.im());
        out[f] = c.norm();
    }

    out
}

/** Fast Fourier Transform - O(nlogn) */
pub fn fft(samples: &[Complex32]) -> Vec<f32> {
    let n = samples.len();
    assert!(
        n.is_power_of_two(),
        "fft: n must be a power of 2, got {}",
        n
    );
    let out = _fft(&samples);
    out.iter().map(|&x| x.norm()).collect()
}

fn _fft(samples: &[Complex32]) -> Vec<Complex32> {
    let n = samples.len();
    if n == 1 {
        return vec![samples[0]];
    }

    let even: Vec<Complex32> = samples.iter().step_by(2).copied().collect();
    let odd: Vec<Complex32> = samples.iter().skip(1).step_by(2).copied().collect();

    let e = _fft(&even);
    let o = _fft(&odd);

    let mut out = vec![Complex32::new(0.0, 0.0); n];

    for f in 0..n / 2 {
        let twiddle = Complex32::new(0.0, -2.0 * PI * f as f32 / n as f32).exp();
        let t = twiddle * o[f];
        out[f] = e[f] + t;
        out[f + n / 2] = e[f] - t;
    }
    out
}
