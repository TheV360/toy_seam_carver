use std::fs::{File, read};
use std::io::{Result as IoResult, Write};

use zune_jpeg::JpegDecoder;
use zune_jpeg::zune_core::{options::DecoderOptions, colorspace::ColorSpace};

mod edge_detect; use edge_detect::*;
mod seam_find; use seam_find::*;

fn main() -> IoResult<()> {
	let args: Vec<String> = std::env::args().skip(1).collect();
	assert_eq!(args.len(), 2, "supply a jpeg file to squish and a target width");
	
	let file = read(&args[0])?;
	let target_width = args[1].parse::<usize>().unwrap();
	
	let mut decoder = JpegDecoder::new_with_options(
		DecoderOptions::default()
			.set_strict_mode(false)
			.jpeg_set_out_colorspace(ColorSpace::RGB),
		&file
	);
	
	let data = decoder.decode().unwrap();
	
	let (width, height) = decoder.dimensions()
		.map(|(w, h)| (w as usize, h as usize))
		.unwrap();
	let size = (width, height);
	
	assert!(target_width <= width, "target width should be less than width");
	let n_seams = width - target_width;
	
	let data = data.chunks_exact(3)
		.map(|c| <&[u8; 3]>::try_from(c).unwrap())
		.cloned()
		.collect::<Vec<_>>();
	
	assert_eq!(data.len(), width * height, "couldn't split data into rgb arrays");
	
	let intensity = rgb_to_intensity(&data, size);
	let edges = edge_detect(&intensity, size);
	let min_energy = min_vert_energy(&edges, size);
	
	write_debug_pgm("intensity", &intensity, size)?;
	write_debug_pgm("edges", &edges, size)?;
	write_debug_pgm("energy", &min_energy, size)?;
	
	eprintln!("finding seams...");
	let seams = find_n_vert_seams(n_seams, &min_energy, size);
	if target_width == 0 {
		// only works if using `find_all_vert_seams`
		eprintln!("visualizing seam order...");
		let seams_ref = seams.iter().map(Vec::as_ref).collect::<Vec<_>>();
		let seams_vis = visualize_seams(&seams_ref);
		write_debug_pgm("seams", &seams_vis, size)?;
	}
	
	eprintln!("writing output image...");
	let mut data = data.to_vec();
	let mut mod_width = width;
	for seam in seams.iter() {
		for (y, x) in seam.iter().cloned().enumerate().rev() {
			data.remove(x + y * mod_width);
		}
		mod_width -= 1;
	}
	write_output_ppm("out", &data, (mod_width, height))?;
	
	Ok(())
}

fn write_debug_pgm(
	filename: &str,
	data: &[f32],
	(width, height): (usize, usize)
) -> IoResult<()> {
	let mut pgm = File::create(format!("./{filename}.pgm"))?;
	
	// write header
	writeln!(pgm, "P5 {width} {height} 255")?;
	
	// get max value to divide by
	let max_val = data.iter().cloned().reduce(f32::max).unwrap_or(1.0);
	
	// map float range to byte range
	let bytes = data.iter()
		.cloned()
		.map(|x| (x * 255.0 / max_val).max(0.0).round() as u8)
		.collect::<Vec<_>>();
	// write it!!!@!
	pgm.write_all(&bytes)?;
	
	eprintln!("wrote debug pgm of {filename}");
	
	Ok(())
}

fn visualize_seams(seams: &[&[usize]]) -> Vec<f32> {
	assert!(!seams.is_empty());
	
	let (width, height) = (seams.len(), seams[0].len());
	
	let mut field = vec![Vec::with_capacity(width); height];
	for (i, seam) in seams.iter().enumerate().rev() {
		for (y, x) in seam.iter().cloned().enumerate() {
			field[y].insert(x, i as f32);
		}
	}
	
	field.into_iter().flatten().collect()
}

fn write_output_ppm(
	filename: &str,
	data: &[[u8; 3]],
	(width, height): (usize, usize)
) -> IoResult<()> {
	let mut ppm = File::create(format!("./{filename}.pgm"))?;
	
	// write header
	writeln!(ppm, "P6 {width} {height} 255")?;
	
	let bytes = data.iter().flatten().cloned().collect::<Vec<_>>();
	ppm.write_all(&bytes)?;
	
	eprintln!("wrote output ppm of {filename}");
	
	Ok(())
}
