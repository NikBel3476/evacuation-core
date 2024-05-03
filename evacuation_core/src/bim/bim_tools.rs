use super::bim_json_object::{BimElementSign, BimJsonObject};
use super::bim_polygon_tools::{is_intersect_line, Line, Polygon};
use super::json_object::Point;
use crate::bim::bim_evac::{
	evac_def_modeling_step, evac_moving_step_test_with_log_rust, get_time_m, get_time_s, time_inc,
	time_reset,
};
use crate::bim::bim_graph::bim_graph_new;
use serde::Serialize;
use std::cmp::Ordering;
use uuid::{uuid, Uuid};

const EVACUATION_MODELING_STEP: f64 = 0.01;
const EVACUATION_MODELING_MAX_SPEED: f64 = 100.0;
const EVACUATION_TIME: f64 = 0.0;

/// Структура, расширяющая элемент DOOR_*
#[derive(Debug, Clone, Default)]
pub struct BimTransit {
	/// UUID идентификатор элемента
	pub uuid: Uuid,
	/// Внутренний номер элемента
	pub id: u64,
	/// Название элемента
	pub name: String,
	/// Массив UUID элементов, которые являются соседними
	pub outputs: Vec<Uuid>,
	/// Полигон элемента
	pub polygon: Polygon,
	/// Высота элемента
	pub size_z: f64,
	/// Уровень, на котором находится элемент
	pub z_level: f64,
	/// Ширина проема/двери
	pub width: f64,
	/// Количество людей, которые прошли через элемент
	pub no_proceeding: f64,
	/// Тип элемента
	pub sign: BimElementSign,
	/// Признак посещения элемента
	pub is_visited: bool,
	/// Признак недоступности элемента для движения
	pub is_blocked: bool,
}

/// Структура, расширяющая элемент типа ROOM и STAIR
#[derive(Debug, Clone, Default)]
pub struct BimZone {
	/// UUID идентификатор элемента
	pub uuid: Uuid,
	/// Внутренний номер элемента
	pub id: u64,
	/// Название элемента
	pub name: String,
	/// Полигон элемента
	pub polygon: Polygon,
	/// Массив UUID элементов, которые являются соседними
	pub outputs: Vec<Uuid>,
	/// Высота элемента
	pub size_z: f64,
	/// Уровень, на котором находится элемент
	pub z_level: f64,
	/// Количество людей в элементе
	pub number_of_people: f64,
	/// Время достижения безопасной зоны
	pub potential: f64,
	/// Площадь элемента
	pub area: f64,
	/// Уровень опасности, % (0, 10, 20, ..., 90, 100)
	pub hazard_level: u8,
	/// Тип элемента
	pub sign: BimElementSign,
	/// Признак посещения элемента
	pub is_visited: bool,
	/// Признак недоступности элемента для движения
	pub is_blocked: bool,
	/// Признак безопасности зоны, т.е. в эту зону возможна эвакуация
	pub is_safe: bool,
}

/// Структура, описывающая этаж
pub struct BimLevel {
	/// Массив зон, которые принадлежат этажу
	pub zones: Vec<BimZone>,
	/// Массив переходов, которые принадлежат этажу
	pub transits: Vec<BimTransit>,
	/// Название этажа
	pub name: String,
	/// Высота этажа над нулевой отметкой
	pub z_level: f64,
}

/// Структура, описывающая здание
pub struct Bim {
	/// Массив уровней здания
	pub levels: Vec<BimLevel>,
	/// Название здания
	pub name: String,
	/// Список зон объекта
	pub zones: Vec<BimZone>,
	/// Список переходов объекта
	pub transits: Vec<BimTransit>,
	/// мин
	pub evacuation_modeling_step: f64,
	/// м/мин
	pub evacuation_modeling_max_speed: f64,
	pub evacuation_time_in_minutes: f64,
}

#[derive(Serialize)]
pub struct EvacuationModelingResult {
	pub number_of_people_inside_building: f64,
	pub number_of_evacuated_people: f64,
	pub time_in_seconds: f64,
	// #[serde(skip)]
	pub people_distribution_stats: Vec<DistributionState>,
	// #[serde(skip)]
	pub distribution_by_time_steps: DistributionByTimeSteps,
}

#[derive(Serialize)]
pub struct DistributionState {
	pub time_in_minutes: f64,
	pub distribution: Vec<f64>,
}

#[derive(Serialize)]
pub struct DistributionByTimeSteps {
	pub items: Vec<ItemTimeStepData>,
}

#[derive(Serialize)]
pub struct ItemTimeStepData {
	// pub doors: Vec<DoorTimeStepData>,
	pub rooms: Vec<RoomTimeStepData>,
	pub time: f64,
}

#[derive(Serialize)]
pub struct DoorTimeStepData {
	pub from: Uuid,
	pub nfrom: f64,
	pub uuid: Uuid,
}

#[derive(Serialize)]
pub struct RoomTimeStepData {
	pub uuid: Uuid,
	pub density: f64,
}

impl Bim {
	pub fn area(&self) -> f64 {
		self.levels.iter().fold(0.0, |acc, level| {
			acc + level.zones.iter().fold(0.0, |acc, zone| match zone.sign {
				BimElementSign::Room | BimElementSign::Staircase => acc + zone.area,
				_ => acc,
			})
		})
	}

	/// Подсчитывает количество людей в здании по расширенной структуре
	pub fn number_of_people(&self) -> f64 {
		self.zones.iter().fold(0.0, |acc, zone| match zone.sign {
			BimElementSign::Outside => acc,
			_ => acc + zone.number_of_people,
		})
	}

	pub fn run_modeling(&mut self) -> EvacuationModelingResult {
		let graph = bim_graph_new(self);

		self.define_modeling_step();
		self.reset_time();

		let remainder = 0.0; // Количество человек, которое может остаться в зд. для остановки цикла
		let mut people_distribution_stats: Vec<DistributionState> =
			vec![self.distributions_statistics()];
		let mut distribution_by_time_steps = DistributionByTimeSteps {
			items: vec![self.items_statistics()],
		};
		loop {
			evac_moving_step_test_with_log_rust(&graph, &mut self.zones, &mut self.transits);
			self.increment_time();
			// bim_output_body(&bim, get_time_m(), &mut fp_detail);
			people_distribution_stats.push(self.distributions_statistics());

			distribution_by_time_steps
				.items
				.push(self.items_statistics());

			if self.number_of_people_in_building() <= remainder {
				break;
			}
		}

		EvacuationModelingResult {
			number_of_people_inside_building: self.number_of_people(),
			number_of_evacuated_people: self.zones[self.zones.len() - 1].number_of_people,
			time_in_seconds: self.get_time_s(),
			people_distribution_stats,
			distribution_by_time_steps,
		}
	}

	fn distributions_statistics(&self) -> DistributionState {
		let mut distribution_stats = vec![];
		for zone in &self.zones {
			distribution_stats.push(zone.number_of_people);
		}
		for transition in &self.transits {
			distribution_stats.push(transition.no_proceeding);
		}

		DistributionState {
			time_in_minutes: self.evacuation_time_in_minutes,
			distribution: distribution_stats,
		}
	}

	fn items_statistics(&self) -> ItemTimeStepData {
		ItemTimeStepData {
			time: self.evacuation_time_in_minutes * 60.0,
			rooms: self
				.zones
				.iter()
				.map(|zone| RoomTimeStepData {
					density: zone.number_of_people,
					uuid: zone.uuid,
				})
				.collect(),
		}
	}

	fn number_of_people_in_building(&self) -> f64 {
		let mut num_of_people = 0.0;
		for zone in &self.zones {
			if zone.is_visited {
				num_of_people += zone.number_of_people;
			}
		}
		num_of_people
	}

	fn define_modeling_step(&mut self) {
		let average_size = self.area() / self.zones.len() as f64;
		let hxy = average_size.sqrt();

		self.evacuation_modeling_step = match self.evacuation_modeling_step.total_cmp(&0.0) {
			Ordering::Equal => hxy / self.evacuation_modeling_max_speed * 0.1,
			_ => self.evacuation_modeling_step,
		}
	}

	fn reset_time(&mut self) {
		self.evacuation_time_in_minutes = 0.0;
	}

	fn get_time_s(&self) -> f64 {
		self.evacuation_time_in_minutes * 60.0
	}

	fn get_time_m(&self) -> f64 {
		self.evacuation_time_in_minutes
	}

	fn increment_time(&mut self) {
		self.evacuation_time_in_minutes += self.evacuation_modeling_step;
	}
}

pub fn intersected_edge(polygon_element: &Polygon, line: &Line) -> Result<Line, String> {
	let mut line_intersected = Line {
		p1: Point { x: 0.0, y: 0.0 },
		p2: Point { x: 0.0, y: 0.0 },
	};

	let mut num_of_intersections = 0;
	for i in 1..polygon_element.points.len() {
		// FIXME: bypass to get double mut ref
		let (left, right) = polygon_element.points.split_at(i);
		let point_element_a = left.last().expect(
			"Failed to get last element of left part at intersected_edge_rust fn in bim_tools crate"
		);
		let point_element_b = right.first().expect(
			"Failed to get first element of right part at intersected_edge_rust fn in bim_tools crate"
		);
		let line_tmp = Line {
			p1: *point_element_a,
			p2: *point_element_b,
		};
		let is_intersected = is_intersect_line(line, &line_tmp);
		if is_intersected {
			line_intersected.p1 = *point_element_a;
			line_intersected.p2 = *point_element_b;
			num_of_intersections += 1;
		}
	}

	if num_of_intersections != 1 {
		return Err(format!("[func: intersected_edge] :: Ошибка геометрии. Проверьте правильность ввода дверей и виртуальных проемов. num_of_intersections: {num_of_intersections}"));
	}

	Ok(line_intersected)
}

/// Возможные варианты стыковки помещений, которые соединены проемом
///
/// Код ниже определяет область их пересечения
/// ```ignore
/// +----+  +----+     +----+
///      |  |               | +----+
///      |  |               | |
///      |  |               | |
/// +----+  +----+          | |
///                         | +----+
/// +----+             +----+
///      |  +----+
///      |  |          +----+ +----+
///      |  |               | |
/// +----+  |               | |
///         +----+          | +----+
///                    +----+
/// ```
/// *************************************************************************
/// 1. Определить грани помещения, которые пересекает короткая сторона проема
/// 2. Вычислить среднее проекций граней друг на друга
pub fn door_way_width(
	zone1: &Polygon,
	zone2: &Polygon,
	edge1: &Line,
	edge2: &Line,
) -> Result<f64, String> {
	// TODO: l1p1 == l2p1 and l1p2 == l2p2 ??? figure out why this is so
	/* old c code
	point_t *l1p1 = edge1->p1;
	point_t *l1p2 = edge2->p2;
	double length1 = geom_tools_length_side_rust( l1p1, l1p2);

	point_t *l2p1 = edge1->p1;
	point_t *l2p2 = edge2->p2;
	double length2 = geom_tools_length_side_rust(l2p1, l2p2);
	 */
	let l1p1 = edge1.p1;
	let l1p2 = edge2.p1;
	let length1 = l1p1.distance_to(&l1p2);

	let l2p1 = edge1.p1;
	let l2p2 = edge2.p2;
	let length2 = l2p1.distance_to(&l2p2);

	// Короткая линия проема, которая пересекает оба помещения
	let d_line = match length1.total_cmp(&length2) {
		Ordering::Greater | Ordering::Equal => Line { p1: l2p1, p2: l2p2 },
		Ordering::Less => Line { p1: l1p1, p2: l1p2 },
	};

	// Линии, которые находятся друг напротив друга и связаны проемом
	let edge_element_a = intersected_edge(zone1, &d_line)?;
	let edge_element_b = intersected_edge(zone2, &d_line)?;

	// Поиск точек, которые являются ближайшими к отрезку edgeElement
	// Расстояние между этими точками и является шириной проема
	let point1 = edge_element_a.p1.nearest_point_on_line(&edge_element_b);
	let point2 = edge_element_a.p2.nearest_point_on_line(&edge_element_b);
	let distance12 = point1.distance_to(&point2);

	let point3 = edge_element_b.p1.nearest_point_on_line(&edge_element_a);
	let point4 = edge_element_b.p2.nearest_point_on_line(&edge_element_a);
	let distance34 = point3.distance_to(&point4);

	Ok((distance12 + distance34) * 0.5)
}

pub fn outside_init_rust(bim_json: &BimJsonObject) -> BimZone {
	let mut outputs: Vec<Uuid> = vec![];
	let mut id = 0u64;

	for level in &bim_json.levels {
		for element in &level.build_elements {
			match element.sign {
				BimElementSign::DoorWayOut => {
					outputs.push(element.uuid);
				}
				BimElementSign::Room | BimElementSign::Staircase => {
					id += 1;
				}
				_ => {}
			}
		}
	}

	if outputs.is_empty() {
		panic!("Failed to find any output at outside_init_rust fn in bim_tools crate")
	}

	BimZone {
		id,
		name: String::from("Outside"),
		sign: BimElementSign::Outside,
		polygon: Polygon::default(),
		uuid: uuid!("00000000-0000-0000-0000-000000000000"),
		z_level: 0.0,
		size_z: f64::from(f32::MAX),
		hazard_level: 0,
		potential: 0.0,
		area: f64::from(f32::MAX),
		outputs,
		is_blocked: false,
		is_visited: false,
		is_safe: true,
		number_of_people: 0.0,
	}
}

/// Вычисление ширины проема по данным из модели здания
///
/// # Parameters:
/// * zones Список всех зон
/// * transits - Список всех переходов
///
/// # Returns
/// Ширина проёма
pub fn calculate_transits_width(zones: &[BimZone], transits: &mut [BimTransit]) {
	for transit in transits {
		let mut stair_sign_counter = 0u8; // Если stair_sign_counter = 2, то проем межэтажный (между лестницами)
		let mut related_zones = [BimZone::default(), BimZone::default()];

		if transit.outputs.is_empty() || transit.outputs.len() > 2 {
			panic!(
				"Transition has {} outputs\n{:#?}",
				transit.outputs.len(),
				transit
			);
		}

		for (i, output) in transit.outputs.iter().enumerate() {
			let zone = zones
				.iter()
				.find(|zone| zone.uuid.eq(output))
				.unwrap_or_else(|| {
					panic!(
						"Failed to find an element connected to transit.\n{:#?}",
						transit
					)
				});

			if zone.sign == BimElementSign::Staircase {
				stair_sign_counter += 1;
			}
			related_zones[i] = zone.clone();
		}

		if stair_sign_counter == 2 {
			// => Межэтажный проем
			transit.width = ((related_zones[0].area + related_zones[1].area) / 2.0).sqrt();
			continue;
		}

		let mut edge1 = Line {
			p1: Point::default(),
			p2: Point::default(),
		};
		let mut edge2 = Line {
			p1: Point::default(),
			p2: Point::default(),
		};
		let mut edge1_number_of_points = 2usize;
		let mut edge2_number_of_points = 2usize;

		for tpoint in &transit.polygon.points {
			let is_point_in_polygon = match related_zones[0].sign {
				BimElementSign::Undefined => false,
				_ => related_zones[0]
					.polygon
					.is_point_inside(tpoint)
					.unwrap_or_else(|msg| {
						panic!("{msg}\n{:#?}\n{:#?}", transit, &related_zones);
					}),
			};

			match is_point_in_polygon {
				true => {
					match edge1_number_of_points {
						2 => edge1.p1 = *tpoint,
						1 => edge1.p2 = *tpoint,
						_ => continue,
					}
					edge1_number_of_points -= 1;
				}
				false => {
					match edge2_number_of_points {
						2 => edge2.p1 = *tpoint,
						1 => edge2.p2 = *tpoint,
						_ => continue,
					}
					edge2_number_of_points -= 1;
				}
			}
		}

		let mut width = -1f64;
		if edge1_number_of_points > 0 || edge2_number_of_points > 0 {
			panic!(
				"Failed to calculate width of transition.\n\
				{:#?}\n\
				{:#?}\n\
				edge1: {edge1_number_of_points}\n\
				edge2: {edge2_number_of_points}",
				transit, &related_zones
			);
		}

		match transit.sign {
			BimElementSign::DoorWayIn | BimElementSign::DoorWayOut => {
				let width1 = edge1.p1.distance_to(&edge1.p2);
				let width2 = edge2.p1.distance_to(&edge2.p2);

				width = (width1 + width2) / 2.0;
			}
			BimElementSign::DoorWay => {
				width = door_way_width(
					&related_zones[0].polygon,
					&related_zones[1].polygon,
					&edge1,
					&edge2,
				)
				.unwrap_or_else(|err_msg| panic!("{err_msg}\n{:#?}", transit));
			}
			_ => {}
		}

		transit.width = width;

		if transit.width < 0.0 {
			panic!(
				"Width of transit is not defined. Transit id: {}, Transit uuid: {}, Transit name: {}, Transit width: {}",
				transit.id,
				transit.uuid,
				transit.name,
				transit.width
			);
		} else if transit.width < 0.5 {
			eprintln!(
				"Warning: Width of transit is less than 0.5. Transit id: {}, Transit uuid: {}, Transit name: {}, Transit width: {}",
				transit.id,
				transit.uuid,
				transit.name,
				transit.width
			);
		}
	}
}

pub fn bim_tools_new_rust(bim_json: &BimJsonObject) -> Bim {
	let mut zones_list: Vec<BimZone> = vec![];
	let mut transits_list: Vec<BimTransit> = vec![];
	let mut levels_list: Vec<BimLevel> = vec![];

	for level_json in &bim_json.levels {
		let mut zones: Vec<BimZone> = vec![];
		let mut transits: Vec<BimTransit> = vec![];

		for build_element_json in &level_json.build_elements {
			let id = build_element_json.id;
			let uuid = build_element_json.uuid;
			let name = build_element_json.name.clone();
			let size_z = build_element_json.size_z;
			let z_level = build_element_json.z_level;
			let sign = build_element_json.sign;
			let outputs = build_element_json.outputs.clone();
			let polygon = build_element_json.polygon.clone();
			let area = polygon.area();

			match build_element_json.sign {
				BimElementSign::Room | BimElementSign::Staircase => {
					let zone = BimZone {
						id,
						uuid,
						name,
						size_z,
						z_level,
						sign,
						// FIXME: unsafe cast u64 to f64
						number_of_people: build_element_json.number_of_people as f64,
						outputs,
						area,
						polygon,
						is_blocked: false,
						is_visited: false,
						is_safe: false,
						potential: f64::from(f32::MAX),
						hazard_level: 0,
					};
					zones.push(zone.clone());
					zones_list.push(zone);
				}
				BimElementSign::DoorWay
				| BimElementSign::DoorWayOut
				| BimElementSign::DoorWayIn => {
					let transit = BimTransit {
						id,
						name,
						uuid,
						size_z,
						z_level,
						sign,
						outputs,
						polygon,
						is_blocked: false,
						is_visited: false,
						no_proceeding: 0.0,
						width: -1.0, // calculate below
					};
					transits.push(transit.clone());
					transits_list.push(transit);
				}
				_ => {}
			}
		}

		let bim_level = BimLevel {
			name: level_json.name.clone(),
			z_level: level_json.z_level,
			zones,
			transits,
		};

		match bim_level.zones.is_empty() || bim_level.transits.is_empty() {
			true => {
				eprintln!(
					"[func: bim_tools_new] :: number of zones ({}) or number of transits ({}) equals zero",
					bim_level.zones.len(),
					bim_level.transits.len()
				);
			}
			false => {}
		}

		levels_list.push(bim_level);
	}

	let outside = outside_init_rust(bim_json);
	zones_list.push(outside);

	zones_list.sort_by(|a, b| a.id.cmp(&b.id));
	transits_list.sort_by(|a, b| a.id.cmp(&b.id));

	calculate_transits_width(&zones_list, &mut transits_list);

	Bim {
		transits: transits_list,
		zones: zones_list,
		levels: levels_list,
		name: bim_json.building_name.clone(),
		evacuation_modeling_step: EVACUATION_MODELING_STEP,
		evacuation_modeling_max_speed: EVACUATION_MODELING_MAX_SPEED,
		evacuation_time_in_minutes: EVACUATION_TIME,
	}
}
