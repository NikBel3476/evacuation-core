use super::bim_tools::Bim;
use crate::bim::bim_tools::DistributionState;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Add;
use std::path::Path;

pub const OUTPUT_DIR: &str = "result";
pub const OUTPUT_DETAIL_FILE_RUST: &str = "_detailed_rust";
pub const OUTPUT_SHORT_FILE_RUST: &str = "_short_rust";
pub const OUTPUT_SUFFIX: &str = ".csv";

pub fn bim_basename_rust(path_to_file: &str) -> String {
	let basename = Path::new(Path::new(path_to_file).file_name().unwrap())
		.file_stem()
		.unwrap()
		.to_str()
		.unwrap()
		.to_owned();

	Path::new("..")
		.join(OUTPUT_DIR)
		.join(basename)
		.to_str()
		.unwrap()
		.to_owned()
}

pub fn bim_create_file_name_rust(base_file_name: &str, middle_name: &str, suffix: &str) -> String {
	Path::new(
		base_file_name
			.to_string()
			.add(middle_name)
			.add(suffix)
			.as_str(),
	)
	.to_str()
	.unwrap()
	.to_owned()
}

pub fn bim_output_head(bim: &Bim, file: &mut File) {
	file.write_all(b"t,").expect("Failed to write to file");

	for zone in &bim.zones {
		file.write_all(format!("{},", zone.name).as_bytes())
			.expect("Failed to write zone name to file");
	}

	for transition in &bim.transits {
		file.write_all(format!("{},", transition.name).as_bytes())
			.expect("Failed to write transition name to file");
	}

	file.write_all(b"\n").expect("Failed to write to file");
	file.flush().expect("Failed to flush file");
}

pub fn bim_output_body(bim: &Bim, time: f64, file: &mut File) {
	file.write_all(format!("{time:.2},").as_bytes())
		.expect("Failed to write to file");

	for zone in &bim.zones {
		file.write_all(format!("{:.2},", zone.number_of_people).as_bytes())
			.expect("Failed to write zone number of people to file");
	}

	for transition in &bim.transits {
		file.write_all(format!("{:.2},", transition.no_proceeding).as_bytes())
			.expect("Failed to write transition no_proceeding to file");
	}

	file.write_all(b"\n").expect("Failed to write to file");
	file.flush().expect("Failed to flush file");
}

pub fn bim_output_body_detailed(distribution_states: &[DistributionState], file: &mut File) {
	let mut bw = BufWriter::new(file);

	for distribution_state in distribution_states {
		bw.write_all(format!("{:.2},", distribution_state.time_in_minutes).as_bytes())
			.expect("Failed to write to file");

		distribution_state
			.distribution
			.iter()
			.for_each(|number_of_people| {
				bw.write_all(format!("{:.2},", number_of_people).as_bytes())
					.expect("Failed to write distribution to file");
			});

		bw.write_all(b"\n").expect("Failed to write to file");
		bw.flush().expect("Failed to flush file");
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::*;

	#[fixture]
	fn path_to_file<'a>() -> &'a str {
		"../res/two_levels.json"
	}

	#[fixture]
	fn expected_path<'a>() -> &'a str {
		if cfg!(unix) {
			"../result/two_levels"
		} else if cfg!(windows) {
			"..\\result\\two_levels"
		} else {
			panic!("This platform is not supported")
		}
	}

	#[rstest]
	fn test_bim_basename_windows(path_to_file: &str, expected_path: &str) {
		let out_file_path = bim_basename_rust(path_to_file);

		assert_eq!(expected_path, out_file_path);
	}
}
