use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::bim::bim_output::{bim_output_body_detailed, OUTPUT_DIR};
use crate::bim::bim_tools::EvacuationModelingResult;
use bim_evac::{
	evac_def_modeling_step, evac_moving_step_test_with_log_rust, get_time_m, get_time_s,
	set_density_max, set_density_min, set_modeling_step, set_speed_max, time_inc, time_reset,
};
use bim_graph::bim_graph_new;
use bim_json_object::{bim_json_object_new, BimElementSign};
use bim_output::{
	bim_basename_rust, bim_create_file_name_rust, bim_output_body, bim_output_head,
	OUTPUT_DETAIL_FILE_RUST, OUTPUT_SHORT_FILE_RUST, OUTPUT_SUFFIX,
};
use bim_tools::{bim_tools_new_rust, Bim};
use cli::CliParameters;
use configuration::{load_cfg, DistributionType, ScenarioCfg, TransitionType};

mod bim_cli;
mod bim_evac;
mod bim_graph;
pub mod bim_json_object;
mod bim_output;
mod bim_polygon_tools;
pub mod bim_tools;
mod cli;
pub mod configuration;
mod graph;
pub mod json_object;
mod json_renga;

pub fn run_rust(scenario_configuration: &ScenarioCfg) {
	let start = Instant::now();
	// TODO: add the logger
	for file in &scenario_configuration.bim_files {
		let filename = bim_basename_rust(file);
		let log_filename = bim_basename_rust("log.txt");

		// Files with results
		let output_detail =
			bim_create_file_name_rust(&filename, OUTPUT_DETAIL_FILE_RUST, OUTPUT_SUFFIX);
		let output_short =
			bim_create_file_name_rust(&filename, OUTPUT_SHORT_FILE_RUST, OUTPUT_SUFFIX);
		let log = bim_create_file_name_rust(&log_filename, "_rust", ".txt");
		let time_data_path = bim_create_file_name_rust(&filename, "_time_data", ".json");

		let mut fp_detail =
			std::fs::File::create(&output_detail).expect("Error opening the output file");
		let mut fp_short =
			std::fs::File::create(&output_short).expect("Error opening the output file");
		let mut log_file = match std::path::Path::new(&log).exists() {
			true => std::fs::File::options()
				.append(true)
				.open(&log)
				.expect("Error opening the log file"),
			false => std::fs::File::create(&log).expect("Error create the log file"),
		};
		let mut time_data_file =
			std::fs::File::create(&time_data_path).expect("Error opening time data file");

		let current_time = chrono::Local::now()
			.format("%Y-%m-%d %H:%M:%S.%6f")
			.to_string();
		let filename_log = format!("The file name of the used bim `{filename}.json`\n");
		print!("{current_time} {filename_log}");
		log_file
			.write_all(filename_log.as_bytes())
			.expect("Failed to write log to file");

		let bim_json = bim_json_object_new(file);

		let mut bim = bim_tools_new_rust(&bim_json);

		applying_scenario_bim_params(&mut bim, scenario_configuration);

		bim_output_head(&bim, &mut fp_detail);

		let modeling_result = bim.run_modeling();

		bim_output_body_detailed(&modeling_result.people_distribution_stats, &mut fp_detail);

		time_data_file
			.write_all(
				serde_json::to_string(&modeling_result.distribution_by_time_steps)
					.expect("Error time data serialization")
					.as_bytes(),
			)
			.expect("Failed to write time data");
		time_data_file.flush().expect("Failed flush time data file");

		let number_of_people_inside_building = modeling_result.number_of_people_inside_building;
		let evacuation_time_m = modeling_result.time_in_seconds / 60.0;
		let evacuated_people = modeling_result.number_of_evacuated_people;

		let evac_time_log = format!(
			"{current_time} Длительность эвакуации: {:.2} с. ({:.2} мин.)\n",
			modeling_result.time_in_seconds,
			modeling_result.time_in_seconds / 60.0
		);
		let number_of_people_log = format!("{current_time} Количество человек: в здании - {number_of_people_inside_building:.2} (в безопасной зоне - {evacuated_people:.2}) чел.\n");
		let delimiter = format!("{current_time} ---------------------------------------\n");

		print!("{evac_time_log}");
		log_file
			.write_all(evac_time_log.as_bytes())
			.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
		print!("{number_of_people_log}");
		log_file
			.write_all(number_of_people_log.as_bytes())
			.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
		print!("{delimiter}");
		log_file
			.write_all(delimiter.as_bytes())
			.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
		log_file
			.flush()
			.unwrap_or_else(|e| panic!("Failed to flush log to file. Error: {e}"));

		fp_short
			.write_all(
				format!(
					"{evacuation_time_m:.2},{number_of_people_inside_building:.2},{evacuated_people:.2}\n"
				)
				.as_bytes(),
			)
			.unwrap_or_else(|e| panic!("Failed to write fp_short to file. Error: {e}"));
		fp_short
			.flush()
			.unwrap_or_else(|e| panic!("Failed to flush fp_short to file. Error: {e}"));
	}
	let end = start.elapsed();
	println!("Completed in {:.2} s", end.as_secs_f64())
}

pub fn run_evacuation_modeling(
	file: &str,
	scenario_configuration: &ScenarioCfg,
) -> EvacuationModelingResult {
	let start = Instant::now();
	// TODO: add the logger
	let filename = bim_basename_rust(file);
	let log_filename = bim_basename_rust("log.txt");

	// Files with results
	let output_detail =
		bim_create_file_name_rust(&filename, OUTPUT_DETAIL_FILE_RUST, OUTPUT_SUFFIX);
	let output_short = bim_create_file_name_rust(&filename, OUTPUT_SHORT_FILE_RUST, OUTPUT_SUFFIX);
	let log = bim_create_file_name_rust(&log_filename, "_rust", ".txt");
	let time_data_path = bim_create_file_name_rust(&filename, "", ".json");
	let result_dir_path = Path::new("..").join(OUTPUT_DIR);
	if std::fs::read_dir(&result_dir_path).is_err() {
		println!("Result directory not found. Creating...");
		std::fs::create_dir(&result_dir_path)
			.unwrap_or_else(|_| panic!("Error creating directory {OUTPUT_DIR}"));
	}

	let mut fp_detail =
		std::fs::File::create(output_detail).expect("Error opening the output file");
	let mut fp_short = std::fs::File::create(output_short).expect("Error opening the output file");
	let mut log_file = match std::path::Path::new(&log).exists() {
		true => std::fs::File::options()
			.append(true)
			.open(&log)
			.expect("Error opening the log file"),
		false => std::fs::File::create(&log).expect("Error create the log file"),
	};
	let mut time_data_file =
		std::fs::File::create(time_data_path).expect("Error opening time data file");

	let current_time = chrono::Local::now()
		.format("%Y-%m-%d %H:%M:%S.%6f")
		.to_string();
	let filename_log = format!("The file name of the used bim `{filename}.json`\n");
	print!("{current_time} {filename_log}");
	log_file
		.write_all(filename_log.as_bytes())
		.expect("Failed to write log to file");

	let bim_json = bim_json_object_new(file);

	let mut bim = bim_tools_new_rust(&bim_json);

	applying_scenario_bim_params(&mut bim, scenario_configuration);

	bim_output_head(&bim, &mut fp_detail);

	let modeling_result = bim.run_modeling();

	bim_output_body_detailed(&modeling_result.people_distribution_stats, &mut fp_detail);

	time_data_file
		.write_all(
			serde_json::to_string_pretty(&modeling_result.distribution_by_time_steps)
				.expect("Error time data serialization")
				.as_bytes(),
		)
		.expect("Failed to write time data");
	time_data_file.flush().expect("Failed flush time data file");

	let number_of_people_inside_building = modeling_result.number_of_people_inside_building;
	let evacuation_time_m = modeling_result.time_in_seconds / 60.0;
	let evacuated_people = modeling_result.number_of_evacuated_people;

	let evac_time_log = format!(
		"{current_time} Длительность эвакуации: {:.2} с. ({:.2} мин.)\n",
		modeling_result.time_in_seconds,
		modeling_result.time_in_seconds / 60.0
	);
	let number_of_people_log = format!("{current_time} Количество человек: в здании - {number_of_people_inside_building:.2} (в безопасной зоне - {evacuated_people:.2}) чел.\n");
	let delimiter = format!("{current_time} ---------------------------------------\n");

	print!("{evac_time_log}");
	log_file
		.write_all(evac_time_log.as_bytes())
		.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
	print!("{number_of_people_log}");
	log_file
		.write_all(number_of_people_log.as_bytes())
		.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
	print!("{delimiter}");
	log_file
		.write_all(delimiter.as_bytes())
		.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
	log_file
		.flush()
		.unwrap_or_else(|e| panic!("Failed to flush log to file. Error: {e}"));

	fp_short
		.write_all(
			format!(
				"{evacuation_time_m:.2},{number_of_people_inside_building:.2},{evacuated_people:.2}\n"
			)
			.as_bytes(),
		)
		.unwrap_or_else(|e| panic!("Failed to write fp_short to file. Error: {e}"));
	fp_short
		.flush()
		.unwrap_or_else(|e| panic!("Failed to flush fp_short to file. Error: {e}"));

	let end = start.elapsed();
	println!("Completed in {:.2} s", end.as_secs_f64());
	modeling_result
}

pub fn run_rust_old() {
	// TODO: remove mock file path
	let cli_parameters = CliParameters {
		scenario_file: String::from("../scenario.json"),
	};

	let scenario_configuration = load_cfg(&cli_parameters.scenario_file)
		.expect("Error reading the scenario configuration file");

	let start = Instant::now();
	// TODO: add the logger
	for file in &scenario_configuration.bim_files {
		let filename = bim_basename_rust(file);
		let log_filename = bim_basename_rust("log.txt");

		// Files with results
		let output_detail =
			bim_create_file_name_rust(&filename, OUTPUT_DETAIL_FILE_RUST, OUTPUT_SUFFIX);
		let output_short =
			bim_create_file_name_rust(&filename, OUTPUT_SHORT_FILE_RUST, OUTPUT_SUFFIX);
		let log = bim_create_file_name_rust(&log_filename, "_rust", ".txt");

		let mut fp_detail =
			std::fs::File::create(&output_detail).expect("Error opening the output file");
		let mut fp_short =
			std::fs::File::create(&output_short).expect("Error opening the output file");
		let mut log_file = match std::path::Path::new(&log).exists() {
			true => std::fs::File::options()
				.append(true)
				.open(&log)
				.expect("Error opening the log file"),
			false => std::fs::File::create(&log).expect("Error create the log file"),
		};

		let current_time = chrono::Local::now()
			.format("%Y-%m-%d %H:%M:%S.%6f")
			.to_string();
		let filename_log = format!("The file name of the used bim `{filename}.json`\n");
		print!("{current_time} {filename_log}");
		log_file
			.write_all(filename_log.as_bytes())
			.expect("Failed to write log to file");

		let bim_json = bim_json_object_new(file);

		let mut bim = bim_tools_new_rust(&bim_json);

		applying_scenario_bim_params(&mut bim, &scenario_configuration);

		bim_output_head(&bim, &mut fp_detail);
		bim_output_body(&bim, 0.0, &mut fp_detail);

		let mut on_modeling_loop_iteration = |bim: &Bim| {
			bim_output_body(bim, get_time_m(), &mut fp_detail);
		};

		run_modeling(&mut bim, &mut on_modeling_loop_iteration);

		let num_of_evacuated_people = bim.number_of_people();
		let evacuation_time_m = get_time_m();
		let evacuated_people = bim.zones[bim.zones.len() - 1].number_of_people;

		let evac_time_log = format!(
			"{current_time} Длительность эвакуации: {:.2} с. ({:.2} мин.)\n",
			get_time_s(),
			get_time_m()
		);
		let number_of_people_log = format!("{current_time} Количество человек: в здании - {num_of_evacuated_people:.2} (в безопасной зоне - {evacuated_people:.2}) чел.\n");
		let delimiter = format!("{current_time} ---------------------------------------\n");

		print!("{evac_time_log}");
		log_file
			.write_all(evac_time_log.as_bytes())
			.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
		print!("{number_of_people_log}");
		log_file
			.write_all(number_of_people_log.as_bytes())
			.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
		print!("{delimiter}");
		log_file
			.write_all(delimiter.as_bytes())
			.unwrap_or_else(|e| panic!("Failed to write log to file. Error: {e}"));
		log_file
			.flush()
			.unwrap_or_else(|e| panic!("Failed to flush log to file. Error: {e}"));

		fp_short
			.write_all(
				format!(
					"{evacuation_time_m:.2},{num_of_evacuated_people:.2},{evacuated_people:.2}\n"
				)
				.as_bytes(),
			)
			.unwrap_or_else(|e| panic!("Failed to write fp_short to file. Error: {e}"));
		fp_short
			.flush()
			.unwrap_or_else(|e| panic!("Failed to flush fp_short to file. Error: {e}"));
	}
	let end = start.elapsed();
	println!("Completed in {:.2} s", end.as_secs_f64())
}

pub fn applying_scenario_bim_params(bim: &mut Bim, scenario_configuration: &ScenarioCfg) {
	for transition in &mut bim.transits {
		if scenario_configuration.transition_parameters.r#type == TransitionType::Users {
			match transition.sign {
				BimElementSign::DoorWayIn => {
					transition.width = scenario_configuration.transition_parameters.doorway_in
				}
				BimElementSign::DoorWayOut => {
					transition.width = scenario_configuration.transition_parameters.doorway_out
				}
				_ => {}
			}
		}

		// A special set up the transit width of item of bim
		for special in &scenario_configuration.transition_parameters.special {
			for uuid in &special.uuid {
				if transition.uuid.eq(uuid) {
					transition.width = special.width;
				}
			}
		}
	}

	// in c code bim->transits is a pointers to bim->levels[_]->transits so necessary to update bim->levels[_]->transits
	for level in &mut bim.levels {
		for transition in &mut level.transits {
			if scenario_configuration.transition_parameters.r#type == TransitionType::Users {
				match transition.sign {
					BimElementSign::DoorWayIn => {
						transition.width = scenario_configuration.transition_parameters.doorway_in
					}
					BimElementSign::DoorWayOut => {
						transition.width = scenario_configuration.transition_parameters.doorway_out
					}
					_ => {}
				}
			}

			// A special set up the transit width of item of bim
			for special in &scenario_configuration.transition_parameters.special {
				for uuid in &special.uuid {
					if transition.uuid.eq(uuid) {
						transition.width = special.width;
					}
				}
			}
		}
	}

	for zone in &mut bim.zones {
		if zone.sign == BimElementSign::Outside {
			continue;
		}

		if scenario_configuration.distribution.r#type == DistributionType::Uniform {
			zone.number_of_people = zone.area * scenario_configuration.distribution.density;
		}

		// A special set up the density of item of bim
		for special in &scenario_configuration.distribution.special {
			for uuid in &special.uuid {
				if zone.uuid.eq(uuid) {
					zone.number_of_people = zone.area * special.density;
				}
			}
		}
	}

	// in c code bim->zones is a pointers to bim->levels[_]->zones so necessary to update bim->levels[_]->zones
	for level in &mut bim.levels {
		for zone in &mut level.zones {
			if zone.sign == BimElementSign::Outside {
				continue;
			}

			if scenario_configuration.distribution.r#type == DistributionType::Uniform {
				zone.number_of_people = zone.area * scenario_configuration.distribution.density;
			}

			// A special set up the density of item of bim
			for special in &scenario_configuration.distribution.special {
				for uuid in &special.uuid {
					if zone.uuid.eq(uuid) {
						zone.number_of_people = zone.area * special.density;
					}
				}
			}
		}
	}

	set_modeling_step(scenario_configuration.modeling_parameters.step);
	set_speed_max(scenario_configuration.modeling_parameters.max_speed);
	set_density_max(scenario_configuration.modeling_parameters.max_density);
	set_density_min(scenario_configuration.modeling_parameters.min_density);

	bim.evacuation_modeling_step = scenario_configuration.modeling_parameters.step;
	bim.evacuation_modeling_max_speed = scenario_configuration.modeling_parameters.max_speed;
}

fn run_modeling(bim: &mut Bim, on_loop_iteration: &mut dyn FnMut(&Bim)) {
	// let graph = bim_graph_new_rust(&bim);
	let graph = bim_graph_new(bim);
	// let bim_graph = bim_graph_new_test(&bim);
	// TODO: add print graph

	evac_def_modeling_step(bim);
	time_reset();

	let remainder = 0.0; // Количество человек, которое может остаться в зд. для остановки цикла
	loop {
		// evac_moving_step_test_with_log(bim_graph, &mut bim.zones, &mut bim.transits);
		evac_moving_step_test_with_log_rust(&graph, &mut bim.zones, &mut bim.transits);
		time_inc();
		// bim_output_body(&bim, get_time_m(), &mut fp_detail);
		on_loop_iteration(bim);

		let mut num_of_people = 0.0;
		for zone in &bim.zones {
			if zone.is_visited {
				num_of_people += zone.number_of_people;
			}
		}

		if num_of_people <= remainder {
			break;
		}
	}
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use insta::assert_yaml_snapshot;
	use rstest::*;
	use serde::Serialize;
	use uuid::uuid;

	use crate::bim::configuration::{
		Distribution, DistributionSpecial, Modeling, Transition, TransitionSpecial,
	};

	use super::*;

	macro_rules! set_snapshot_suffix {
		($($expr:expr),*) => {
			let mut settings = insta::Settings::clone_current();
			settings.set_snapshot_suffix(format!($($expr,)*));
			let _guard = settings.bind_to_scope();
		}
	}

	#[derive(Serialize)]
	struct ModelingResult {
		number_of_people_inside_building: f64,
		number_of_evacuated_people: f64,
		time_in_seconds: f64,
	}

	impl From<EvacuationModelingResult> for ModelingResult {
		fn from(value: EvacuationModelingResult) -> Self {
			Self {
				number_of_people_inside_building: value.number_of_people_inside_building,
				number_of_evacuated_people: value.number_of_evacuated_people,
				time_in_seconds: value.time_in_seconds,
			}
		}
	}

	#[fixture]
	fn scenario_configuration() -> ScenarioCfg {
		ScenarioCfg {
			version: String::new(),
			bim_files: vec![],
			logger_cfg: String::new(),
			distribution: Distribution {
				r#type: DistributionType::Uniform,
				density: 0.1,
				special: vec![DistributionSpecial {
					uuid: vec![uuid!("87c49613-44a7-4f3f-82e0-fb4a9ca2f46d")],
					density: 1.0,
					comment: String::new(),
				}],
			},
			transition_parameters: Transition {
				r#type: TransitionType::FromBim,
				doorway_in: 0.0,
				doorway_out: 0.0,
				special: vec![TransitionSpecial {
					uuid: vec![uuid!("dcbd8b6e-6dd0-4583-8aac-2492797f8032")],
					width: 1.5,
					comment: String::new(),
				}],
			},
			modeling_parameters: Modeling {
				step: 0.01,
				max_speed: 100.0,
				min_density: 0.1,
				max_density: 5.0,
			},
		}
	}

	// #[test]
	// fn test_run_modeling() {
	// 	run_rust();
	// }

	// #[rstest]
	// #[case::example_one_exit(scenario_configuration(), "../res/example-one-exit.json")]
	// #[case::example_two_exits(scenario_configuration(), "../res/example-two-exits.json")]
	// #[case::one_zone_one_exit(scenario_configuration(), "../res/one_zone_one_exit.json")]
	// #[case::three_zone_three_transit(
	// 	scenario_configuration(),
	// 	"../res/three_zone_three_transit.json"
	// )]
	// #[case::two_levels(scenario_configuration(), "../res/two_levels.json")]
	// #[case::building_test(scenario_configuration(), "../res/building_test.json")]
	// #[case::udsu_b1_L4_v2_190701(scenario_configuration(), "../res/udsu_b1_L4_v2_190701.json")]
	// #[case::udsu_b2_L4_v1_190701(scenario_configuration(), "../res/udsu_b2_L4_v1_190701.json")]
	// #[case::udsu_b3_L3_v1_190701(scenario_configuration(), "../res/udsu_b3_L3_v1_190701.json")]
	// #[case::udsu_b4_L5_v1_190701(scenario_configuration(), "../res/udsu_b4_L5_v1_190701.json")]
	// #[case::udsu_b5_L4_v1_200102(scenario_configuration(), "../res/udsu_b5_L4_v1_200102.json")]
	// #[case::udsu_b7_L8_v1_190701(scenario_configuration(), "../res/udsu_b7_L8_v1_190701.json")]
	// fn modeling(#[case] scenario_configuration: ScenarioCfg, #[case] file_path: &str) {
	// 	let bim_json = bim_json_object_new(file_path);
	// 	let mut bim = bim_tools_new_rust(&bim_json);
	//
	// 	applying_scenario_bim_params(&mut bim, &scenario_configuration);
	//
	// 	let mut on_modeling_loop_iteration = |_: &Bim| {};
	//
	// 	run_modeling(&mut bim, &mut on_modeling_loop_iteration);
	//
	// 	let modeling_result = ModelingResult {
	// 		number_of_people_in_building: bim.number_of_people(),
	// 		evacuation_time_in_seconds: get_time_s(),
	// 		number_of_evacuated_people: bim.zones[bim.zones.len() - 1].number_of_people,
	// 	};
	//
	// 	let file_name = Path::new(file_path).file_stem().unwrap().to_str().unwrap();
	// 	set_snapshot_suffix!("{file_name}");
	// 	assert_yaml_snapshot!(modeling_result);
	// }

	#[rstest]
	#[case::example_one_exit(scenario_configuration(), "../res/example-one-exit.json")]
	// #[case::example_two_exits(scenario_configuration(), "../res/example-two-exits.json")]
	// #[case::one_zone_one_exit(scenario_configuration(), "../res/one_zone_one_exit.json")]
	// #[case::three_zone_three_transit(
	// 	scenario_configuration(),
	// 	"../res/three_zone_three_transit.json"
	// )]
	// #[case::two_levels(scenario_configuration(), "../res/two_levels.json")]
	// #[case::building_test(scenario_configuration(), "../res/building_test.json")]
	// #[case::udsu_block_1(scenario_configuration(), "../res/udsu_b1_L4_v2_190701.json")]
	// #[case::udsu_block_1_old(scenario_configuration(), "../res/b1-v2.1_with_corridors_v1.1.json")]
	// #[case::udsu_block_2(scenario_configuration(), "../res/udsu_b2_L4_v1_190701.json")]
	// #[case::udsu_block_3(scenario_configuration(), "../res/udsu_b3_L3_v1_190701.json")]
	// #[case::udsu_block_4(scenario_configuration(), "../res/udsu_b4_L5_v1_190701.json")]
	// #[case::udsu_block_5(scenario_configuration(), "../res/udsu_b5_L4_v1_200102.json")]
	// #[case::udsu_block_7(scenario_configuration(), "../res/udsu_b7_L8_v1_190701.json")]
	fn evacuation_modeling(
		#[case] mut scenario_configuration: ScenarioCfg,
		#[case] file_path: &str,
		#[values(0.1, 0.2, 0.3, 0.4, 0.5, 1.0, 2.0, 3.0, 4.0)] density: f64,
	) {
		let bim_json = bim_json_object_new(file_path);
		let mut bim = bim_tools_new_rust(&bim_json);

		scenario_configuration.distribution.density = density;
		applying_scenario_bim_params(&mut bim, &scenario_configuration);

		let modeling_result = ModelingResult::from(bim.run_modeling());

		let file_name = Path::new(file_path).file_stem().unwrap().to_str().unwrap();
		set_snapshot_suffix!("{file_name}-density-{density:.1}");
		assert_yaml_snapshot!(modeling_result);
	}
}
