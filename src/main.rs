mod raymod;
use raymod::*;

use rayon::prelude::*;
use std::f64::consts::*;
use std::io::Write;


fn radiance(r: &Ray, depth: u8, ef: f64,scene:&Scene) -> Vec3 {
    let mut t: f64 = 0.0;
    let mut id = 0;
    if !scene.intersect(r, &mut t, &mut id) {
        return Vec3::zero();
    }
    let obj = &scene.objects[id];
    let x = r.o + r.d * t;
    let n = (x - obj.p).norm();
    let nl = if n.dot(&r.d) < 0.0 { n } else { n * -1.0 };
    let mut f = obj.c;
    let p = if f.x > f.y && f.x > f.z {
        f.x
    } else if f.y > f.z {
        f.y
    } else {
        f.z
    };
    let depth = depth + 1;
    if depth > 5 {
        if random() < p {
            f = f * (1.0 / p);
        } else {
            return obj.e * ef;
        }
    }

    return match obj.refl {
        Refl::Diff => {
            let r1 = 2.0 * PI * random();
            let r2 = random();
            let r2s = r2.sqrt();
            let w = nl;
            let u = ((if w.x.abs() > 0.1 {Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) }) % w).norm();
            let v = w % u;
            let d = (u * f64::cos(r1) * r2s + v * f64::sin(r1) * r2s + w * (1.0 - r2).sqrt()).norm();
            // Loop over any lights
            let mut em = Vec3::zero();
            let tid = id;
            for i in 0..scene.objects.len() {
                let s = &scene.objects[i];
                if i == tid {continue;};
                if s.e.x <= 0.0 && s.e.y <= 0.0 && s.e.z <= 0.0 {continue;}; // skip non-lights
                let sw = s.p - x;
                let light_rate = s.rad * s.rad / sw.length();
                let su = ((if sw.x.abs() > 0.1 {Vec3::new(0.0, 1.0, 0.0)} else {Vec3::new(1.0, 0.0, 0.0)}) % sw).norm();
                let sv = sw % su;
                if light_rate > 1.0 {
                    let eps1 = 2.0 * PI * random();
                    let eps2 = random();
                    let eps2s = (1.0 - eps2).sqrt();
                    let ss = eps1.sin();
                    let cc = eps1.cos();
                    let l = (u * (cc * eps2s) + v * (ss * eps2s) + w * (1.0 - eps2).sqrt()).norm();
                    if scene.intersect(&Ray { o: x, d: l }, &mut t, &mut id) == true && id == i {
                        let n_rate = l.dot(&nl);
                        em = em + f.mult(&(s.e * n_rate)) // 1/pi for brdf
                    }
                } else {
                    let cos_a_max = (1.0 - light_rate).sqrt();
                    let eps1 = random();
                    let eps2 = random();
                    let cos_a = 1.0 - eps1 + eps1 * cos_a_max;
                    let sin_a = (1.0 - cos_a * cos_a).sqrt();
                    let phi = 2.0 * PI * eps2;
                    let mut l: Vec3 =
                        su * (phi).cos() * sin_a + sv * (phi).sin() * sin_a + sw * cos_a;
                    l = l.norm();
                    if scene.intersect(&Ray { o: x, d: l }, &mut t, &mut id) == true && id == i {
                        // shadow ray
                        let omega = 2.0 * PI * (1.0 - cos_a_max);
                        let mut n_rate = l.dot(&nl);
                        //if n_rate< 0.0 || n_rate>1.0 {
                        //    println!("n_rate={} ---- l.x={} l.y={} l.z={}  ---- nl.x={} nl.y={} nl.z={}",n_rate,l.x,l.y,l.z,nl.x,nl.y,nl.z);
                        //}
                        if n_rate < 0.0 { n_rate = 0.0 };
                        em = em + f.mult(&(s.e * n_rate * omega)) * FRAC_1_PI; // 1/pi for brdf
                    }
                }
            } //end for
            obj.e * ef + em + f.mult(&radiance(&Ray::new(x, d), depth, 0.0,scene))
        }
        Refl::Spec => {
            obj.e + f.mult(&radiance( &Ray::new(x, r.d - n * 2.0 * n.dot(&r.d)), depth,1.0,scene))
        }
        _ => {
            // Refl.Refr
            let refl_ray = Ray::new(x, r.d - n * 2.0 * n.dot(&r.d));
            let into = n.dot(&nl) > 0.0;
            let nc = 1.0;
            let nt = 1.5;
            let nnt = if into { nc / nt } else { nt / nc };
            let ddn = r.d.dot(&nl);
            let cos2t = 1.0 - nnt * nnt * (1.0 - ddn * ddn);
            if cos2t < 0.0 {
                obj.e + f.mult(&radiance(&refl_ray, depth, 1.0,scene))
            } else {
                let tdir =
                    r.d * nnt - n * ((if into { 1.0 } else { -1.0 }) * (ddn * nnt + cos2t.sqrt()));
                tdir.norm();
                let a = nt - nc;
                let b = nt + nc;
                let r0 = a * a / (b * b);
                let c = 1.0 - (if into { -ddn } else { tdir.dot(&n) });
                let re = r0 + (1.0 - r0) * c * c * c * c * c;
                let tr = 1.0 - re;
                let p = 0.25 + 0.5 * re;
                let rp = re / p;
                let tp = tr / (1.0 - p);
                obj.e
                    + f.mult(
                        &(if depth > 2 {
                            if random() < p {
                                radiance(&refl_ray, depth, 1.0,scene) * rp
                            } else {
                                radiance(&Ray::new(x, tdir), depth, 1.0,scene) * tp
                            }
                        } else {
                            radiance(&refl_ray, depth, 1.0,scene) * re
                                + radiance(&Ray::new(x, tdir), depth, 1.0,scene) * tr
                        }),
                    )
            }
        }
    };
}

fn main() {

    let args = parameters();
    println!("{:?}", args);
    
	let mut scene=Scene::init();
    match args.m{
        0=> scene.model_init0(),
        1=> scene.model_init1(),
        2=> scene.model_init2(),
        3=> scene.model_init3(),
        4=> scene.model_init4(),
        5=> scene.model_init5(),
        6=> scene.model_init6(),
        7=> scene.model_init7(),
        8=> scene.model_init8(),
        9=> scene.model_init9(),
        _=> scene.model_init0(),
    };

	
    let w: usize = args.w;
    let h: usize = (480.0*w as f64/640.0) as usize;
    let samps = args.s;

    let cam = Ray::new(
        Vec3::new(50.0, 52.0, 295.6),
        Vec3::new(0.0, -0.042612, -1.0).norm(),
    );

    let cx = Vec3::new((w as f64) * 0.5135 / (h as f64), 0.0, 0.0);
    let cy = (cx % cam.d).norm() * 0.5135;
    let mut image = vec![Color::zero(); (w * h) as usize];

    let bands: Vec<(usize, &mut [Color])> = image.chunks_mut(w as usize).enumerate().collect();
    bands.into_par_iter().for_each(|(y, band)| {
        let y2 = h - (y as usize) - 1;
        if (y % 10) == 0 {
            writeln!( std::io::stderr(), "Rendering ({} spp) {:5.2}%", samps * 4,100.0 * (y as f64) / ((h as f64) - 1.0) ).unwrap();
        }
        for x in 0..w {
            let mut r = Vec3::zero();
            for sy in 0..2 {
                for sx in 0..2 {
                    for _s in 0..samps {
                        let r1 = 2.0 * random();
                        let dx = if r1 < 1.0 { r1.sqrt() - 1.0 } else { 1.0 - (2.0 - r1).sqrt() };
                        let r2 = 2.0 * random();
                        let dy = if r2 < 1.0 { r2.sqrt() - 1.0 } else { 1.0 - (2.0 - r2).sqrt() };
                        let d = cx * ((((sx as f64) + 0.5 + dx) / 2.0 + (x as f64)) / (w as f64) - 0.5)
                              + cy * ((((sy as f64) + 0.5 + dy) / 2.0 + (y2 as f64)) / (h as f64)- 0.5)
                              + cam.d;
                        r = r + radiance(&(Ray::new(cam.o + d * 140.0, d.norm())), 0, 1.0,&scene) * (1.0 / (samps as f64));
                    }
                    band[x as usize] = band[x as usize] + r * (1.0 / 4.0 as f64);
                    r = Vec3::zero();
                }
            }
        }
    });

    //    save_ppm_file("image.ppm", image, w, h);
    save_png_file(&args.output, image, w, h);
}
