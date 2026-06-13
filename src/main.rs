use num_complex::Complex;

type C = Complex<f64>;

// A point in the Poincaré disk (|z| < 1)
#[derive(Debug, Clone, Copy)]
struct DiskPoint(C);

impl DiskPoint {
    fn new(x: f64, y: f64) -> Option<DiskPoint> {
        let z = C::new(x, y);
        if z.norm() < 1.0 {
            Some(DiskPoint(z))
        } else {
            None
        }
    }
}

// Möbius transformation: f(z) = (az + b) / (cz + d)
fn mobius(a: C, b: C, c: C, d: C, z: C) -> C {
    (a * z + b) / (c * z + d)
}

// Disk isometry: f(z) = (z + w) / (1 + w.conj() * z)
// Translates the origin to w
fn disk_isometry(z: C, w: C) -> C {
    (z + w) / (C::new(1.0, 0.0) + w.conj() * z)
}

fn rotate(z: C, theta: f64) -> C {
    let rotation = C::new(theta.cos(), theta.sin());
    rotation * z
}

fn geodesic_points(z1: C, z2: C, samples: usize) -> Vec<C> {
    // Special case: if both points are on the same diameter, it's a straight line
    let cross = z1.re * z2.im - z1.im * z2.re;
    if cross.abs() < 1e-10 {
        return (0..samples)
            .map(|i| {
                let t = i as f64 / (samples - 1) as f64;
                z1 * (1.0 - t) + z2 * t
            })
            .collect();
    }

    // Find the center of the geodesic circle
    // It lies on the perpendicular bisector of z1-z2 and outside the unit disk
    let mid = (z1 + z2) * C::new(0.5, 0.0);
    
    // The center has to satisfy |c - z1|² = |c - z2|² = r²
    // and |c|² - r² = 1 (orthogonal to unit circle)
    let z1_norm_sq = z1.norm_sqr();
    let z2_norm_sq = z2.norm_sqr();

    // Solve for center (cx, cy)
    let ax = 2.0 * (z2.re - z1.re);
    let ay = 2.0 * (z2.im - z1.im);
    let bx = 2.0 * z1.re;
    let by = 2.0 * z1.im;
    let c1 = z2_norm_sq - z1_norm_sq;
    let c2 = z1_norm_sq - 1.0;

    let det = ax * by - ay * bx;
    let cx = (c1 * by - ay * c2) / det;
    let cy = (ax * c2 - c1 * bx) / det;
    let center = C::new(cx, cy);
    let radius = (z1 - center).norm();

    // Sample points along the arc from z1 to z2
    let angle1 = (z1 - center).im.atan2((z1 - center).re);
    let angle2 = (z2 - center).im.atan2((z2 - center).re);

    // Go the short way around
    let mut delta = angle2 - angle1;
    if delta > std::f64::consts::PI { delta -= 2.0 * std::f64::consts::PI; }
    if delta < -std::f64::consts::PI { delta += 2.0 * std::f64::consts::PI; }

    (0..samples)
        .map(|i| {
            let t = i as f64 / (samples - 1) as f64;
            let angle = angle1 + t * delta;
            center + C::new(radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}

fn render_svg(points: &[(&str, C)], output_path: &str) {
    let size = 500.0_f64;
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r = size / 2.0 - 10.0;

    let bg = "#1a1a2e";
    let disk_fill = "#16213e";
    let accent = "#e94560";
    let axis = "#ffffff22";

    let mut doc = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}">
<rect width="{size}" height="{size}" fill="{bg}"/>
<circle cx="{cx}" cy="{cy}" r="{r}" fill="{disk_fill}" stroke="{accent}" stroke-width="2"/>
<line x1="{}" y1="{cy}" x2="{}" y2="{cy}" stroke="{axis}" stroke-width="1"/>
<line x1="{cx}" y1="{}" x2="{cx}" y2="{}" stroke="{axis}" stroke-width="1"/>"#,
        cx - r, cx + r, cy - r, cy + r
    );

    for (label, z) in points {
        let px = cx + z.re * r;
        let py = cy - z.im * r;
        doc.push_str(&format!(
            r#"<circle cx="{px:.2}" cy="{py:.2}" r="6" fill="{accent}"/>
<text x="{:.2}" y="{:.2}" fill="white" font-size="14" font-family="monospace">{label}</text>"#,
            px + 8.0, py - 8.0
        ));
    }

    // Draw geodesics between all pairs
    let pairs = vec![
        (0, 5), // p -> a
        (0, 6), // p -> b
        (5, 6), // a -> b
        (5, 7), // a -> c
        (6, 8), // b -> d
        (7, 8), // c -> d
        (8, 9), // d -> e
        (5, 9), // a -> e
        (6, 7), // b -> c
    ];
    
    for (i, j) in pairs {
        if i < points.len() && j < points.len() {
            let geo = geodesic_points(points[i].1, points[j].1, 100);
            let path: String = geo.iter().enumerate().map(|(k, z)| {
                let px = cx + z.re * r;
                let py = cy - z.im * r;
                if k == 0 { format!("M {:.2} {:.2}", px, py) }
                else { format!("L {:.2} {:.2}", px, py) }
            }).collect::<Vec<_>>().join(" ");
            doc.push_str(&format!(
                r#"<path d="{}" stroke="{accent}" stroke-width="1.5" fill="none" opacity="0.6"/>"#,
                path
            ));
        }
    }

    doc.push_str("\n</svg>");
    std::fs::write(output_path, doc).unwrap();
    println!("SVG written to {}", output_path);
}

fn main() {
    let origin = DiskPoint::new(0.0, 0.0).unwrap();
    let p = DiskPoint::new(0.5, 0.0).unwrap();
    let w = C::new(0.3, 0.0);

    println!("=== Möbius Transforms ===");
    println!("Original point p: {:?}", p.0);
    
    let moved = disk_isometry(p.0, w);
    println!("After isometry (translate by 0.3): {:.4} + {:.4}i", moved.re, moved.im);
    println!("Still in disk: {}", moved.norm() < 1.0);

    // Apply isometry to origin
    let moved_origin = disk_isometry(origin.0, w);
    println!("\nOrigin moved to: {:.4} + {:.4}i", moved_origin.re, moved_origin.im);

    // Rotation by 45 degrees
    let theta = std::f64::consts::PI / 4.0;
    let rotated = rotate(p.0, theta);
    println!("\n=== Rotation ===");
    println!("p rotated 45°: {:.4} + {:.4}i", rotated.re, rotated.im);
    println!("Norm preserved: {:.4}", rotated.norm());

    // Compose: translate then rotate
    println!("\n=== Composition ===");
    let translated = disk_isometry(p.0, w);
    let then_rotated = rotate(translated, theta);
    println!("p translated then rotated 45°: {:.4} + {:.4}i", then_rotated.re, then_rotated.im);
    println!("Still in disk: {}", then_rotated.norm() < 1.0);
    println!("Norm: {:.4}", then_rotated.norm());

    // Reverse order: rotate then translate
    let rotated_first = rotate(p.0, theta);
    let then_translated = disk_isometry(rotated_first, w);
    println!("\np rotated 45° then translated: {:.4} + {:.4}i", then_translated.re, then_translated.im);
    println!("Still in disk: {}", then_translated.norm() < 1.0);
    println!("Norm: {:.4}", then_translated.norm());

    // Show isometry as explicit Möbius transform
    // disk_isometry(z, w) = (z + w) / (1 + conj(w) * z)
    // which is (az+b)/(cz+d) where a=1, b=w, c=conj(w), d=1
    println!("\n=== General Möbius ===");
    let a = C::new(1.0, 0.0);
    let b = w;
    let c = w.conj();
    let d = C::new(1.0, 0.0);
    let mobius_result = mobius(a, b, c, d, p.0);
    println!("Isometry via disk_isometry:  {:.4} + {:.4}i", translated.re, translated.im);
    println!("Isometry via mobius:         {:.4} + {:.4}i", mobius_result.re, mobius_result.im);
    println!("Same result: {}", (mobius_result - translated).norm() < 1e-10);

    // Show rotation as explicit Möbius transform
    // rotate(z, theta) = e^(i*theta) * z
    // which is (az+b)/(cz+d) where a=e^(i*theta), b=0, c=0, d=1
    let rot = C::new(theta.cos(), theta.sin());
    let mobius_rotation = mobius(rot, C::new(0.0, 0.0), C::new(0.0, 0.0), C::new(1.0, 0.0), p.0);
    println!("\nRotation via rotate:   {:.4} + {:.4}i", rotated.re, rotated.im);
    println!("Rotation via mobius:   {:.4} + {:.4}i", mobius_rotation.re, mobius_rotation.im);
    println!("Same result: {}", (mobius_rotation - rotated).norm() < 1e-10);

    let svg_points = vec![
        ("p", p.0),
        ("translated", translated),
        ("rotated", rotated),
        ("T+R", then_rotated),
        ("R+T", then_translated),
        // New points spread around the disk
        ("a", C::new(-0.5, 0.0)),
        ("b", C::new(0.0, 0.5)),
        ("c", C::new(-0.3, -0.6)),
        ("d", C::new(0.6, -0.4)),
        ("e", C::new(-0.7, 0.3)),
    ];

    render_svg(&svg_points, "disk.svg");
}