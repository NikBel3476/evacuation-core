#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use tauri::{WindowBuilder, AppHandle};

use cli;
use configuration;
use json_object;
use bim_cli;
use bim_configure;
use bim_evac;
use bim_json_object;
use bim_output;

mod run_bindings;

fn main() {
	tauri::Builder::default()
		.invoke_handler(
			tauri::generate_handler![
				read_config,
				open_configuration_window,
				open_configuration_rescript_window,
				open_people_traffic_window,
				bim_start
			]
		)
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

#[tauri::command]
fn read_config() -> Result<configuration::ScenarioCfg, String> {
	configuration::load_cfg("../scenario.json")
}

#[tauri::command]
async fn open_configuration_window(handle: AppHandle) {
	let _configuration_window = WindowBuilder::new(
        &handle,
        "configuration",
        tauri::WindowUrl::App("src-ui/config/index.html".into())
	).build().unwrap();
}

#[tauri::command]
async fn open_configuration_rescript_window(handle: AppHandle) {
	let _configuration_window = WindowBuilder::new(
		&handle,
		"configurationRescript",
		tauri::WindowUrl::App("src-ui/configRescript/index.html".into())
	).build().unwrap();
}

#[tauri::command]
async fn open_people_traffic_window(handle: AppHandle) {
	let _people_traffic_window = WindowBuilder::new(
		&handle,
		"people_traffic",
		tauri::WindowUrl::App("src-ui/peopleTraffic/index.html".into())
	)
	.min_inner_size(1000.0, 800.0)
	.build()
	.unwrap();
}

#[tauri::command]
async fn bim_start() {
	unsafe { run_bindings::run() };
}