use std::ffi::CString;
use std::os::raw::c_char;

#[derive(Debug)]
#[repr(C)]
pub struct cli_params_t {
	scenario_file: *const c_char,
}

#[no_mangle]
pub extern "C" fn read_cl_args() -> Box<cli_params_t> {
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

	let string = CString::new(args[1].clone()).unwrap();
	let ptr = string.as_ptr();

	std::mem::forget(string);

	let params = cli_params_t { scenario_file: ptr };

	Box::new(params)
}
