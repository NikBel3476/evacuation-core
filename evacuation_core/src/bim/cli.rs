use std::error::Error;

pub struct CliParameters {
	pub scenario_file: String,
}

pub fn read_cli_args() -> Result<CliParameters, Box<dyn Error>> {
	let args: Vec<String> = std::env::args().collect();

	if args.len() < 2 {
		println!("Ожидаются аргументы");
		std::process::exit(0);
	}

	if args[1] == "--help" || args[1] == "-h" {
		println!(
			"Использование: {} <путь к файлу сценария моделирования>",
			args[0]
		);
		std::process::exit(0);
	}

	Ok(CliParameters {
		scenario_file: args[1].clone(),
	})
}
