use std::error::Error;
use std::fs;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildingStructRenga {
	#[serde(rename = "nameBuilding")]
	pub name: String,
	pub program_name: String,
	#[serde(rename = "version_program")]
	pub program_version: String,
	#[serde(rename = "numberOfRoom")]
	pub number_of_rooms: i32,
	#[serde(rename = "numberOfDoor")]
	pub number_of_doors: i32,
	#[serde(rename = "numberofDoorWayInt")]
	pub number_of_inside_doorways: i32,
	#[serde(rename = "numberOfDoorWayOut")]
	pub number_of_outside_doorways: i32,
	#[serde(rename = "numberOfStaircase")]
	pub number_of_staircases: i32,
	#[serde(rename = "date_creation_Json")]
	pub json_creating_date: String,
	#[serde(rename = "address_building")]
	pub address: AddressRenga,
	#[serde(rename = "Level")]
	pub levels: Vec<BuildingLevelRenga>,
	#[serde(rename = "Devs")]
	pub devs: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AddressRenga {
	#[serde(rename = "city")]
	pub city: String,
	#[serde(rename = "streetAddress")]
	pub street_address: String,
	#[serde(rename = "addInfo")]
	pub add_info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildingLevelRenga {
	#[serde(rename = "NameLevel")]
	pub name: String,
	#[serde(rename = "ZLevel")]
	pub z_level: f64,
	#[serde(rename = "BuildElement")]
	pub building_elements: Vec<BuildingElementRenga>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildingElementRenga {
	#[serde(rename = "Name")]
	pub name: String,
	#[serde(rename = "Id")]
	pub uuid: Uuid,
	#[serde(rename = "Sign")]
	pub sign: String,
	#[serde(rename = "SizeZ")]
	pub size_z: f64,
	// #[serde(rename = "Wide")]
	// pub width: f64,
	#[serde(rename = "Output")]
	pub outputs: Vec<Uuid>,
}

impl BuildingStructRenga {
	pub fn parse_building_from_json(
		path_to_file: &str,
	) -> Result<Box<BuildingStructRenga>, Box<dyn Error>> {
		let json_content = fs::read_to_string(path_to_file).unwrap_or_else(|err| {
			panic!(
				"Ошибка чтения файла конфигурации здания {}: {}",
				path_to_file, err
			);
		});

		let data: BuildingStructRenga = serde_json::from_str(&json_content).unwrap_or_else(|err| {
			panic!(
				"Ошибка десериализации файла конфигурации здания {}: {}",
				path_to_file, err
			);
		});

		Ok(Box::new(data))
	}
}
