use std::fs::File;
use std::io::Write;

use zune_jpeg::JpegDecoder;
use zune_jpeg::zune_core::{options::DecoderOptions, colorspace::ColorSpace};

mod edge_detect; use edge_detect::*;
mod seam_find; use seam_find::*;

fn main() -> std::io::Result<()> {
	let args: Vec<String> = std::env::args().skip(1).collect();
	assert_eq!(args.len(), 1, "supply a jpeg file to squish");
	
	let file = std::fs::read(&args[0])?;
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
	
	let data = data.chunks_exact(3)
		.map(|c| <&[u8; 3]>::try_from(c).unwrap())
		.cloned()
		.collect::<Vec<_>>();
	
	assert_eq!(data.len(), width * height, "couldn't split data into rgb arrays");
	
	let intensity = rgb_to_intensity(&data, size);
	let edges = edge_detect(&intensity, size);
	let min_energy = min_vert_energy(&edges, size);
	
	write_debug_ppm("intensity", &intensity, size)?;
	write_debug_ppm("edges", &edges, size)?;
	write_debug_ppm("energy", &min_energy, size)?;
	
    find_vert_seam(&min_energy, size);
	
	Ok(())
}

fn write_debug_ppm(
	filename: &str,
	data: &[f32],
	(width, height): (usize, usize)
) -> std::io::Result<()> {
	let mut ppm = File::create(format!("./{filename}.ppm"))?;
	
	// write header
	writeln!(ppm, "P5 {width} {height} 255")?;
	
	// get max value to divide by
	let max_val = data.iter().cloned().reduce(f32::max).unwrap_or(1.0);
	
	// map float range to byte range
	let bytes = data.iter()
		.cloned()
		.map(|x| (x * 255.0 / max_val).max(0.0).round() as u8)
		.collect::<Vec<_>>();
	// write it!!!@!
	ppm.write_all(&bytes)?;
	
	eprintln!("wrote debug ppm of {filename}");
	
	Ok(())
}
