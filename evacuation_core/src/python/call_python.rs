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

		let python_exec =
			PyModule::from_code(py, code.as_str(), "generateJSON.py", "generateJSON")?;

		python_exec.getattr("main")?.call0().unwrap();
		Ok(())
	})
}
