use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum DistributionType {
	FromBim,
	Uniform,
}

impl std::fmt::Display for DistributionType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			DistributionType::FromBim => "FromBim",
			DistributionType::Uniform => "Uniform",
		};

		write!(f, "{text}")
	}
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TransitionType {
	FromBim,
	Users,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DistributionSpecial {
	pub uuid: Vec<Uuid>,
	pub density: f64,
	pub comment: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransitionSpecial {
	pub uuid: Vec<Uuid>,
	pub width: f64,
	pub comment: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Distribution {
	pub r#type: DistributionType,
	pub density: f64,
	pub special: Vec<DistributionSpecial>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
	pub r#type: TransitionType,
	pub doorway_in: f64,
	pub doorway_out: f64,
	pub special: Vec<TransitionSpecial>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Modeling {
	pub step: f64,
	pub max_speed: f64,
	pub max_density: f64,
	pub min_density: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioCfg {
	pub version: String,
	pub bim_files: Vec<String>,
	pub logger_cfg: String,
	pub distribution: Distribution,
	pub transition_parameters: Transition,
	pub modeling_parameters: Modeling,
}

// impl Default for ScenarioCfg {
// 	fn default() -> Self {
// 		Self {
// 			version: String::from("1.0"),

// 		}
// 	}
// }

pub fn load_cfg(path_to_file: &str) -> Result<ScenarioCfg, String> {
	match Path::new(path_to_file).exists() {
		true => {
			let json_content = fs::read_to_string(path_to_file).unwrap_or_else(|err| {
				panic!(
					"Ошибка чтения файла конфигурации сценария {}: {}",
					path_to_file, err
				)
			});

			let cfg: ScenarioCfg = serde_json::from_str(&json_content).unwrap_or_else(|err| {
				panic!(
					"Ошибка десериализации файла конфигурации сценария {}: {}",
					path_to_file, err
				)
			});

			Ok(cfg)
		}
		false => Err(format!("Не удалось найти указанный файл: {}", path_to_file)),
	}
}

pub fn save_configuration<P: AsRef<Path>>(
	path_to_file: P,
	configuration: &ScenarioCfg,
) -> Result<(), Box<dyn Error>> {
	let writer = BufWriter::new(File::create(path_to_file)?);
	serde_json::to_writer_pretty(writer, configuration)?;
	Ok(())
}
