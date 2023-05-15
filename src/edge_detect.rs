const KERNEL_SOBEL_X: &[[f32; 3]; 3] = &[
	[ -1./8.,  0./1.,  1./8. ],
	[ -1./4.,  0./1.,  1./4. ],
	[ -1./8.,  0./1.,  1./8. ]
];
const KERNEL_SOBEL_Y: &[[f32; 3]; 3] = &[
	[ -1./8., -1./4., -1./8., ],
	[  0./1.,  0./1.,  0./1., ],
	[  1./8.,  1./4.,  1./8., ]
];

pub fn intensity_from_rgb([r, g, b]: [u8; 3]) -> f32 {
	// from RGB -> YUV conversion
	0.299 * (r as f32 / 255.0) +
	0.587 * (g as f32 / 255.0) +
	0.114 * (b as f32 / 255.0)
}

pub fn rgb_to_intensity(
	rgb: &[[u8; 3]], (width, height): (usize, usize)
) -> Vec<f32> {
	assert!(width  > 0, "need non-zero width");
	assert!(height > 0, "need non-zero height");
	
	assert_eq!(
		rgb.len(), width * height,
		"need color data array to match width*height"
	);
	
	rgb.iter().cloned().map(intensity_from_rgb).collect()
}

fn kernel_mul<const S: usize>(
	kernel: &[[f32; S]; S], the: [[f32; S]; S]
) -> f32 {
	the.iter().flatten()
		.zip(kernel.iter().flatten())
		.map(|(t, k)| t * k)
		.sum()
}

pub fn edge_detect(
	intensity: &[f32], (width, height): (usize, usize)
) -> Vec<f32> {
	assert!(width  > 0, "need non-zero width");
	assert!(height > 0, "need non-zero height");
	
	assert_eq!(
		intensity.len(), width * height,
		"need intensity array to match width*height"
	);
	
	let mut edginess = intensity.to_vec();
	
	for y in 0..height {
		let (y_prev, y_next) = (
			if y >          0 { y.saturating_sub(1) } else {          0 },
			if y < height - 1 { y.saturating_add(1) } else { height - 1 },
		);
		for x in 0..width {
			let (x_prev, x_next) = (
				if x >         0 { x.saturating_sub(1) } else {         0 },
				if x < width - 1 { x.saturating_add(1) } else { width - 1 },
			);
			
			let surroundings = [
				[ (x_prev, y_prev), (x, y_prev), (x_next, y_prev) ],
				[ (x_prev, y     ), (x, y     ), (x_next, y     ) ],
				[ (x_prev, y_next), (x, y_next), (x_next, y_next) ],
			].map(|row| row
				.map(|(x, y)| intensity[y * width + x]));
			
			edginess[y * width + x] = f32::sqrt(
				f32::powi(kernel_mul(KERNEL_SOBEL_X, surroundings), 2) +
				f32::powi(kernel_mul(KERNEL_SOBEL_Y, surroundings), 2)
			);
		}
	}
	
	edginess
}
