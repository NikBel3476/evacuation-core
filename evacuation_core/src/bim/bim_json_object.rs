use super::bim_polygon_tools;
use crate::bim::json_object::BuildingStruct;
use crate::bim::json_renga::BuildingStructRenga;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BimElementSign {
	/// Указывает, что элемент здания является помещением/комнатой
	Room,
	/// Указывает, что элемент здания является лестницей
	Staircase,
	/// Указывает, что элемент здания является проемом (без дверного полотна)
	DoorWay,
	/// Указывает, что элемент здания является дверью, которая соединяет два элемента: ROOM и ROOM или ROOM и STAIR
	DoorWayIn,
	/// Указывает, что элемент здания является эвакуационным выходом
	DoorWayOut,
	/// Указывает, что элемент является зоной вне здания
	Outside,
	/// Указывает, что тип элемента не определен
	#[default]
	Undefined,
}

/// Структура, описывающая элемент
#[derive(Debug)]
pub struct BimJsonElement {
	/// [JSON] UUID идентификатор элемента
	pub uuid: Uuid,
	/// [JSON] Название элемента
	pub name: String,
	/// [JSON] Полигон элемента
	pub polygon: bim_polygon_tools::Polygon,
	/// [JSON] Массив UUID элементов, которые являются соседними к элементу
	pub outputs: Vec<Uuid>,
	/// Внутренний номер элемента (генерируется)
	pub id: u64,
	/// [JSON] Количество людей в элементе
	pub number_of_people: u64,
	/// [JSON] Высота элемента
	pub size_z: f64,
	/// Уровень, на котором находится элемент
	pub z_level: f64,
	/// [JSON] Тип элемента
	pub sign: BimElementSign,
}

/// Структура поля, описывающего географическое положение объекта
pub struct BimJsonAddress {
	/// [JSON] Название города
	pub city: String,
	/// [JSON] Название улицы
	pub street_address: String,
	/// [JSON] Дополнительная информация о местоположении объекта
	pub add_info: String,
}

/// Структура, описывающая этаж
pub struct BimJsonLevel {
	/// [JSON] Название этажа
	pub name: String,
	/// [JSON] Массив элементов, которые принадлежат этажу
	pub build_elements: Vec<BimJsonElement>,
	/// [JSON] Высота этажа над нулевой отметкой
	pub z_level: f64,
}

/// Структура, описывающая здание
pub struct BimJsonObject {
	/// [JSON] Информация о местоположении объекта
	pub address: BimJsonAddress,
	/// [JSON] Название здания
	pub building_name: String,
	/// [JSON] Массив уровней здания
	pub levels: Vec<BimJsonLevel>,
}

pub fn bim_json_object_new(path_to_file: &str) -> BimJsonObject {
	// TODO: remove renga search in path
	let building = match path_to_file.contains("renga") {
		true => Box::new(BuildingStruct::from(
			BuildingStructRenga::parse_building_from_json(path_to_file)
				.unwrap_or_else(|e| panic!("Failed to parse building. Error: {e}"))
				.as_ref(),
		)),
		false => BuildingStruct::parse_building_from_json(path_to_file)
			.unwrap_or_else(|e| panic!("Failed to parse building. Error: {e}")),
	};
	let mut bim_element_rs_id: u64 = 0;
	let mut bim_element_d_id: u64 = 0;

	let json_object = BimJsonObject {
		address: BimJsonAddress {
			city: building.address.city,
			street_address: building.address.street_address,
			add_info: building.address.add_info,
		},
		building_name: building.building_name,
		levels: building
			.levels
			.iter()
			.map(|level| BimJsonLevel {
				name: level.name.clone(),
				z_level: level.z_level,
				build_elements: level
					.build_elements
					.iter()
					.map(|element| BimJsonElement {
						uuid: element.id,
						name: element.name.clone(),
						id: match element.sign.as_str() {
							"Room" | "Staircase" => {
								let id = bim_element_rs_id;
								bim_element_rs_id += 1;
								id
							}
							"DoorWay" | "DoorWayInt" | "DoorWayOut" => {
								let id = bim_element_d_id;
								bim_element_d_id += 1;
								id
							}
							element_type => {
								panic!("Неизвестный тип элемента здания: {}", element_type)
							}
						},
						size_z: element.size_z,
						z_level: level.z_level,
						number_of_people: element.number_of_people,
						sign: match element.sign.as_str() {
							"Room" => BimElementSign::Room,
							"Staircase" => BimElementSign::Staircase,
							"DoorWay" => BimElementSign::DoorWay,
							"DoorWayInt" => BimElementSign::DoorWayIn,
							"DoorWayOut" => BimElementSign::DoorWayOut,
							_ => BimElementSign::Undefined,
						},
						outputs: element.outputs.clone(),
						polygon: bim_polygon_tools::Polygon::from(match element.xy.is_empty() {
							true => &[],
							false => element.xy[0].points.as_slice(),
						}),
					})
					.collect::<Vec<BimJsonElement>>(),
			})
			.collect::<Vec<BimJsonLevel>>(),
	};

	json_object
}
