use raylib::ffi::{atan2f, sinf};

mod fourier_transform {
    use num::complex::Complex32;
    use std::f32::consts::PI;

    // fn main() {
    //     const N: usize = 8;
    //
    //     let mut inputs: [f32; N] = [0.0; N];
    //     for i in 0..N {
    //         let t = i as f32 / N as f32;
    //         // sinwave of 1Hz + coswave of 2Hz
    //         inputs[i] = f32::sin(2.0 * PI * t * 1.0) + f32::cos(2.0 * PI * t * 2.0);
    //     }
    //
    //     let out = dft(&inputs);
    //     for f in 0..N {
    //         println!("dft_out[{}] {:.2}", f, out[f]);
    //     }
    //
    //     println!();
    //
    //     let mut _inputs: Vec<Complex32> = inputs.iter().map(|&x| Complex32::new(x, 0.0)).collect();
    //     let _out = fft(&_inputs);
    //     for f in 0..N {
    //         println!("fft_out[{}] {:.2}", f, _out[f].norm());
    //     }
    // }

    /** Descrete Fourier Transform O(n^2)*/
    fn dft(samples: &[f32]) -> Vec<f32> {
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
    fn fft(samples: &[Complex32]) -> Vec<Complex32> {
        let n = samples.len();
        if n == 1 {
            return vec![samples[0]];
        }

        assert!(n % 2 == 0, "fft: n must be a power of 2, got {}", n);

        let even: Vec<Complex32> = samples.iter().step_by(2).copied().collect();
        let odd: Vec<Complex32> = samples.iter().skip(1).step_by(2).copied().collect();

        let e = fft(&even);
        let o = fft(&odd);

        let mut out = vec![Complex32::new(0.0, 0.0); n];

        for f in 0..n / 2 {
            let twiddle = Complex32::new(0.0, -2.0 * PI * f as f32 / n as f32).exp();
            let t = twiddle * o[f];
            out[f] = e[f] + t;
            out[f + n / 2] = e[f] - t;
        }
        out
    }
}
