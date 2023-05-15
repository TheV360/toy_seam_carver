use std::cmp::Ordering;

pub fn min_vert_energy(
	energy: &[f32], (width, height): (usize, usize)
) -> Box<[f32]> {
	assert!(width  > 0, "need non-zero width");
	assert!(height > 0, "need non-zero height");
	
	assert_eq!(
		energy.len(), width * height,
		"need energy array to match width*height"
	);
	
	let mut min_energy = energy.to_vec().into_boxed_slice();
	
	let mut rows = min_energy.chunks_exact_mut(width).rev();
	
	// bottom row is starting point, so we skip it.
	// wish that there were windows for iterators... but this works too!
	let mut prev_row = rows.next().unwrap();
	for row in rows {
		for (col, cell) in row.iter_mut().enumerate() {
			*cell += [
				prev_row[col.saturating_sub(1)],
				prev_row[col],
				prev_row[Ord::min(col.saturating_add(1), width - 1)],
			].into_iter().reduce(f32::min).unwrap();
		}
		prev_row = row;
	}
	
	min_energy
}

/// Finds the seam using a pre-calculated min. energy array.
pub fn find_vert_seam(
	min_energy: &[f32], (width, height): (usize, usize)
) -> Vec<usize> {
	assert!(width  > 0, "need non-zero width");
	assert!(height > 0, "need non-zero height");
	
	assert_eq!(
		min_energy.len(), width * height,
		"need energy array to match width*height"
	);
	
	// construct result
	let mut result = Vec::with_capacity(height);
	
	fn get_min_index<T: PartialOrd>(data: &[T]) -> usize {
		data.iter()
			.enumerate()
			.reduce(|(pi, pv), (ni, nv)|
				match pv.partial_cmp(nv) {
					Some(Ordering::Less) => (pi, pv),
					_                    => (ni, nv),
				}
			).unwrap().0
	}
	
	let mut rows = min_energy.chunks_exact(width);
	
	let mut prev_index = get_min_index(rows.next().unwrap());
	result.push(prev_index);
	
	for row in rows {
		// to find seam, get path of least resistance down `min_energy`.
		// we found the starting index before we entered this loop, now
		// we just need to follow it down and remember what indices we used.
		
		// search in `prev_index + (-1..=1)`,
		// but clamp if it'd index out of bounds.
		let range_lo = prev_index.saturating_sub(1);
		let range_hi = Ord::min(prev_index.saturating_add(1), width - 1);
		let range_slice = &row[range_lo..=range_hi];
		
		// since the slice we're handing `get_min_index` doesn't have
		// a starting index, we have to re-add the starting index
		// to the value it returns.
		prev_index = get_min_index(range_slice) + range_lo;
		result.push(prev_index);
	}
	
	result
}

/// Finds `n` vertical seams in the given image.
/// 
/// The indices for seam `i + 1` will assume you've
/// already removed seam `i` from the picture.
pub fn find_n_vert_seams(
	n: usize,
	min_energy: &[f32], (width, height): (usize, usize)
) -> Vec<Vec<usize>> {
	assert!(width  > 0, "need non-zero width");
	assert!(height > 0, "need non-zero height");
	
	assert_eq!(
		min_energy.len(), width * height,
		"need energy array to match width*height"
	);
	
	assert!(
		n <= width,
		"not enough columns in image for specified number of seams"
	);
	
	let mut seams = Vec::with_capacity(n);
	let mut mod_energy = min_energy.to_vec();
	
	for m in 0..n {
		// TODO: make this more efficient by like. making a generic 2d array
		//  interface and etc. passing in a Arr2D<f32> into find_seam...
		//  and then it uses .get(x, y) and you can easily swap in a
		//  jagged array instead of memcopying so fucking much.
		// who cares about monomorphization bloat. i don't acre.
		// (it doesn't matter at this scale, i'm just a shaky creature)
		
		let width = width - m;
		
		// get seam
		let seam = find_vert_seam(&mod_energy, (width, height));
		let seam_indices = seam.iter()
			.cloned().enumerate()
			.map(|(y, x)| y * width + x);
		
		// remove seam
		for i in seam_indices.rev() {
			mod_energy.remove(i);
		}
		
		// remember seam
		seams.push(seam);
	}
	
	seams
}

pub fn find_all_vert_seams(
	min_energy: &[f32], (width, height): (usize, usize)
) -> Vec<Vec<usize>> {
	find_n_vert_seams(width, min_energy, (width, height))
}
