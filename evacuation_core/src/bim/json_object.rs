use super::bim_polygon_tools::Line;
use crate::bim::json_renga::{
	AddressRenga, BuildingElementRenga, BuildingLevelRenga, BuildingStructRenga,
};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Address {
	#[serde(rename = "City")]
	pub city: String,
	#[serde(rename = "StreetAddress")]
	pub street_address: String,
	#[serde(rename = "AddInfo")]
	pub add_info: String,
}

impl From<AddressRenga> for Address {
	fn from(address_renga: AddressRenga) -> Self {
		Self {
			city: address_renga.city,
			street_address: address_renga.street_address,
			add_info: address_renga.add_info,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct Point {
	pub x: f64,
	pub y: f64,
}

impl Point {
	pub fn distance_to(&self, other: &Point) -> f64 {
		let x = self.x - other.x;
		let y = self.y - other.y;
		(x * x + y * y).sqrt()
	}

	/// Определение точки на линии, расстояние до которой от заданной точки является минимальным из существующих
	pub fn nearest_point_on_line(&self, line: &Line) -> Point {
		let p1 = &line.p1;

		let p2 = &line.p2;

		if p1.distance_to(p2) < 1e-9 {
			return line.p1;
		}

		let a = self.x - p1.x;
		let b = self.y - p1.y;
		let c = p2.x - p1.x;
		let d = p2.y - p1.y;

		let dot = a * c + b * d;
		let len_sq = c * c + d * d;
		let mut param = -1.0;

		if len_sq != 0.0 {
			param = dot / len_sq;
		}

		let xx;
		let yy;

		if param < 0.0 {
			xx = p1.x;
			yy = p1.y;
		} else if param > 1.0 {
			xx = p2.x;
			yy = p2.y;
		} else {
			xx = p1.x + param * c;
			yy = p1.y + param * d;
		}

		Point { x: xx, y: yy }
	}
}

// impl From<Point> for iced::Point {
// 	fn from(value: Point) -> Self {
// 		Self {
// 			x: value.x as f32,
// 			y: value.y as f32,
// 		}
// 	}
// }

// impl From<&Point> for iced::Point {
// 	fn from(value: &Point) -> Self {
// 		Self {
// 			x: value.x as f32,
// 			y: value.y as f32,
// 		}
// 	}
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinates {
	pub points: Vec<Point>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildElement {
	#[serde(rename = "Id")]
	pub id: Uuid,
	#[serde(rename = "@")]
	pub uuid: Uuid,
	#[serde(rename = "Name")]
	pub name: String,
	#[serde(rename = "SizeZ")]
	pub size_z: f64,
	#[serde(rename = "Sign")]
	pub sign: String,
	#[serde(rename = "XY")]
	pub xy: Vec<Coordinates>,
	#[serde(rename = "Output")]
	pub outputs: Vec<Uuid>,
	#[serde(rename = "NumPeople", default)]
	pub number_of_people: u64,
}

impl From<&BuildingElementRenga> for BuildElement {
	fn from(element_renga: &BuildingElementRenga) -> Self {
		let mut rng = thread_rng();
		let pivot_point = Point {
			x: rng.gen_range(0.0..=10.0),
			y: rng.gen_range(0.0..=10.0),
		};
		Self {
			outputs: element_renga.outputs.clone(),
			// TODO: replace random with real coordinates
			xy: vec![Coordinates {
				points: vec![
					pivot_point,
					Point {
						x: pivot_point.x + 1.0,
						..pivot_point
					},
					Point {
						x: pivot_point.x + 1.0,
						y: pivot_point.y - 1.0,
					},
					Point {
						y: pivot_point.y - 1.0,
						..pivot_point
					},
					pivot_point,
				],
			}],
			// TODO: implement
			number_of_people: 0,
			size_z: element_renga.size_z,
			sign: element_renga.sign.clone(),
			// TODO: implement
			id: element_renga.uuid,
			uuid: element_renga.uuid,
			name: element_renga.name.clone(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Level {
	#[serde(rename = "NameLevel")]
	pub name: String,
	#[serde(rename = "ZLevel")]
	pub z_level: f64,
	#[serde(rename = "BuildElement")]
	pub build_elements: Vec<BuildElement>,
}

impl From<&BuildingLevelRenga> for Level {
	fn from(level_renga: &BuildingLevelRenga) -> Self {
		Self {
			name: level_renga.name.clone(),
			z_level: level_renga.z_level,
			build_elements: level_renga
				.building_elements
				.iter()
				.map(BuildElement::from)
				.collect(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildingStruct {
	#[serde(rename = "Devs")]
	pub devs: Vec<i64>,
	#[serde(rename = "NameBuilding")]
	pub building_name: String,
	#[serde(rename = "Address")]
	pub address: Address,
	#[serde(rename = "Level")]
	pub levels: Vec<Level>,
}

impl BuildingStruct {
	pub fn parse_building_from_json(
		path_to_file: &str,
	) -> Result<Box<BuildingStruct>, Box<dyn Error>> {
		let json_content = fs::read_to_string(path_to_file).unwrap_or_else(|err| {
			panic!(
				"Ошибка чтения файла конфигурации здания {}: {}",
				path_to_file, err
			);
		});

		let data: BuildingStruct = serde_json::from_str(&json_content).unwrap_or_else(|err| {
			panic!(
				"Ошибка десериализации файла конфигурации здания {}: {}",
				path_to_file, err
			);
		});

		Ok(Box::new(data))
	}
}

impl From<&BuildingStructRenga> for BuildingStruct {
	fn from(building_struct_renga: &BuildingStructRenga) -> Self {
		Self {
			address: Address::from(building_struct_renga.address.clone()),
			devs: building_struct_renga.devs.clone(),
			building_name: building_struct_renga.name.clone(),
			levels: building_struct_renga
				.levels
				.iter()
				.map(Level::from)
				.collect(),
		}
	}
}
