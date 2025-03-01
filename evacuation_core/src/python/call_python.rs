use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::fs::File;
use std::io::Read;

pub fn run_python() -> PyResult<()> {
	Python::with_gil(|py| {
		let mut file =
			File::open("../scripts/generateJSON.py").expect("Failed to open python file");
		let mut code = String::new();
		file.read_to_string(&mut code).unwrap();

		let code = std::ffi::CString::new(code).expect("Cannon convert code to C string");
		let file_name = std::ffi::CString::new("generateJSON.py")
			.expect("Cannot convert file name to C string");
		let module_name =
			std::ffi::CString::new("generateJSON").expect("Cannot convert module name to C string");
		let python_exec = PyModule::from_code(
			py,
			code.as_c_str(),
			file_name.as_c_str(),
			module_name.as_c_str(),
		)?;

		python_exec.getattr("main")?.call0().unwrap();
		Ok(())
	})
}
