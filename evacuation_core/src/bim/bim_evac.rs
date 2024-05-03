use super::bim_graph::BimGraph;
use super::bim_json_object::BimElementSign;
use super::bim_tools::{Bim, BimTransit, BimZone};
use std::cmp::Ordering;

/// м/мин
static mut EVAC_SPEED_MAX_RUST: f64 = 100.0;
/// чел/м^2
static mut EVAC_DENSITY_MIN_RUST: f64 = 0.1;
/// чел/м^2
static mut EVAC_DENSITY_MAX_RUST: f64 = 5.0;
/// мин
static mut EVAC_MODELING_STEP_RUST: f64 = 0.01;
static mut EVAC_TIME_RUST: f64 = 0.0;

// TODO: change parameters naming
/// Функция скорости. Базовая зависимость, которая позволяет определить скорость
/// людского потока по его плотности
///
/// # Arguments
/// * `v0` - начальная скорость потока
/// * `a` - коэффициент вида пути
/// * `d` - текущая плотность людского потока на участке, чел./м2
/// * `d0` - допустимая плотность людского потока на участке, чел./м2
///
/// # Returns
/// скорость, м/мин.
fn velocity_rust(v0: f64, a: f64, d: f64, d0: f64) -> f64 {
	v0 * (1.0 - a * (d / d0).ln())
}

/// Скорость потока в проёме
///
/// # Arguments
/// * `transit_width` - ширина проема, м
/// * `density_in_zone` - плотность в элементе, чел/м2
/// * `v_max` - максимальная скорость потока
///
/// # Returns
/// скорость потока в проеме в зависимости от плотности, м/мин
pub fn speed_through_transit_rust(transit_width: f64, density_in_zone: f64, v_max: f64) -> f64 {
	let v0 = v_max;
	let d0 = 0.65;
	let a = 0.295;

	// TODO: add logging if v0k < 0
	match density_in_zone > d0 {
		true => {
			let m = match density_in_zone > 5.0 {
				true => 1.25 - 0.05 * density_in_zone,
				false => 1.0,
			};

			if density_in_zone >= 9.0 && transit_width < 1.6 {
				return 10.0 * (2.5 + 3.75 * transit_width) / d0;
			}

			velocity_rust(v0, a, density_in_zone, d0) * m
		}
		false => v0,
	}
}

/// Скорость потока в комнате
///
/// # Arguments
/// * `density_in_zone` - плотность в элементе, из которого выходит поток
/// * `v_max` - максимальная скорость потока
///
/// # Returns
/// Скорость потока по горизонтальному пути, м/мин
pub fn speed_in_room_rust(density_in_zone: f64, v_max: f64) -> f64 {
	let d0 = 0.51;

	match density_in_zone > d0 {
		true => velocity_rust(v_max, 0.295, density_in_zone, d0),
		false => v_max,
	}
}

/// Скорость потока на лестнице
///
/// # Arguments
/// * `density_in_zone` - плотность в элементе, из которого выходит поток
/// * `direction` - направление движения (1 - вверх, -1 - вниз)
///
/// # Returns
/// Скорость потока при движении по лестнице в зависимости от плотности, м/мин
pub fn evac_speed_on_stair_rust(density_in_zone: f64, direction: i32) -> f64 {
	let mut d0: f64 = 0.0;
	let mut v0: f64 = 0.0;
	let mut a: f64 = 0.0;

	match direction.cmp(&0) {
		Ordering::Greater => {
			d0 = 0.67;
			v0 = 50.0;
			a = 0.305;
		}
		Ordering::Less => {
			d0 = 0.89;
			v0 = 80.0;
			a = 0.4;
		}
		Ordering::Equal => {}
	}

	match density_in_zone > d0 {
		true => velocity_rust(v0, a, density_in_zone, d0),
		false => v0,
	}
}

/// Метод определения скорости движения людского потока по разным зонам
///
/// # Arguments
/// * `receiving_zone` - зона, в которую засасываются люди
/// * `transmitting_zone` - зона, из которой высасываются люди
///
/// # Returns
/// Скорость людского потока в зоне
pub fn speed_in_element(receiving_zone: &BimZone, transmitting_zone: &BimZone) -> f64 {
	let density_in_transmitting_zone = transmitting_zone.number_of_people / transmitting_zone.area;
	// По умолчанию, используется скорость движения по горизонтальной поверхности
	let mut v_zone =
		unsafe { speed_in_room_rust(density_in_transmitting_zone, EVAC_SPEED_MAX_RUST) };
	// Разница высот зон
	let dh = receiving_zone.z_level - transmitting_zone.z_level;

	// Если принимающее помещение является лестницей и находится на другом уровне,
	// то скорость будет рассчитываться как по наклонной поверхности
	if dh.abs() > 1e-3 && receiving_zone.sign == BimElementSign::Staircase {
		/* Иначе определяем направление движения по лестнице
		 * -1 вниз, 1 вверх
		 *         ______   aGiverItem
		 *        /                         => direction = -1
		 *       /
		 * _____/           aReceivingItem
		 *      \
		 *       \                          => direction = 1
		 *        \______   aGiverItem
		 */
		let direction = if dh > 0.0 { -1 } else { 1 };
		v_zone = evac_speed_on_stair_rust(density_in_transmitting_zone, direction);
	}

	// TODO: Add logging
	// if v_zone < 0 { log() }

	v_zone
}

/// Определение скорости на выходе из отдающего помещения
///
/// # Arguments
/// * `receiving_zone` - принимающая зона
/// * `transmitting_zone` - отдающая зона
/// * `transit_width` - ширина прохода
///
/// # Returns
/// Скорость на выходе из отдающего помещения
pub fn speed_at_exit(
	receiving_zone: &BimZone,
	transmitting_zone: &BimZone,
	transit_width: f64,
) -> f64 {
	let zone_speed = speed_in_element(receiving_zone, transmitting_zone);
	let density_in_transmitting_element =
		transmitting_zone.number_of_people / transmitting_zone.area;
	let transition_speed = unsafe {
		speed_through_transit_rust(
			transit_width,
			density_in_transmitting_element,
			EVAC_SPEED_MAX_RUST,
		)
	};

	zone_speed.min(transition_speed)
}

// TODO: complete docs comment
///
///
/// # Arguments
/// * `transmitting_zone` - отдающая зона
/// * `transit_width` - ширина прохода
/// * `speed_at_exit` - Скорость перехода в принимающую зону
///
/// # Returns
///
pub fn change_num_of_people(
	transmitting_zone: &BimZone,
	transit_width: f64,
	speed_at_exit: f64,
) -> f64 {
	let density_in_element = transmitting_zone.number_of_people / transmitting_zone.area;
	// Величина людского потока, через проем, чел./мин
	let people_flow = density_in_element * speed_at_exit * transit_width;
	// Зная скорость потока, можем вычислить конкретное количество человек,
	// которое может перейти в принимющую зону (путем умножения потока на шаг моделирования)
	unsafe { people_flow * EVAC_MODELING_STEP_RUST }
}

// TODO: Уточнить корректность подсчета потенциала
// TODO: Потенциал должен считаться до эвакуации из помещения или после?
// TODO: Когда возникает ситуация, что потенциал принимающего больше отдающего
/// Подсчет потенциала
///
/// # Arguments
/// * `receiving_zone` - принимающая зона
/// * `transmitting_zone` - отдающая зона
/// * `transit` - проем
///
/// # Returns
/// Потенциал
pub fn potential_element(
	receiving_zone: &BimZone,
	transmitting_zone: &BimZone,
	transit: &BimTransit,
) -> f64 {
	let p = transmitting_zone.area.sqrt()
		/ speed_at_exit(receiving_zone, transmitting_zone, transit.width);

	match receiving_zone.potential.total_cmp(&f64::from(f32::MAX)) {
		Ordering::Less => receiving_zone.potential + p,
		_ => p,
	}
}

/// part_people_flow
///
/// # Arguments
/// * `receiving_zone` - принимающее помещение
/// * `transmitting_zone` - отдающее помещение
/// * `transit` - проем между помещениями
///
/// # Returns
/// Количество людей
pub fn part_people_flow(
	receiving_zone: &BimZone,
	transmitting_zone: &BimZone,
	transit: &BimTransit,
) -> f64 {
	let area_transmitting_zone = transmitting_zone.area;
	let people_in_transmitting_zone = transmitting_zone.number_of_people;
	let density_in_transmitting_zone = people_in_transmitting_zone / area_transmitting_zone;
	let density_min_transmitting_zone = unsafe {
		match EVAC_DENSITY_MIN_RUST > 0.0 {
			true => EVAC_DENSITY_MIN_RUST,
			false => 0.5 / area_transmitting_zone,
		}
	};

	// Ширина перехода между зонами зависит от количества человек,
	// которое осталось в помещении. Если там слишком мало людей,
	// то они переходят все сразу, чтоб не дробить их
	let door_width = transit.width; //(densityInElement > densityMin) ? aDoor.VCn().getWidth() : std::sqrt(areaElement);
	let speed_at_exit = speed_at_exit(receiving_zone, transmitting_zone, door_width);

	// Количество людей, которые могут покинуть помещение
	let part_of_people_flow = match density_in_transmitting_zone > density_min_transmitting_zone {
		true => change_num_of_people(transmitting_zone, door_width, speed_at_exit),
		false => people_in_transmitting_zone,
	};

	// Т.к. зона вне здания принята безразмерной,
	// в нее может войти максимально возможное количество человек
	// Все другие зоны могут принять ограниченное количество человек.
	// Т.о. нужно проверить может ли принимающая зона вместить еще людей.
	// capacity_receiving_zone - количество людей, которое еще может
	// вместиться до достижения максимальной плотности
	// => если может вместить больше, чем может выйти, то вмещает всех вышедших,
	// иначе вмещает только возможное количество.
	let max_num_of_people = unsafe { EVAC_DENSITY_MAX_RUST * receiving_zone.area };
	let capacity_receiving_zone = max_num_of_people - receiving_zone.number_of_people;

	// Такая ситуация возникает при плотности в принимающем помещении более Dmax чел./м2
	// Фактически capacity_receiving_zone < 0 означает, что помещение не может принять людей
	if capacity_receiving_zone < 0.0 {
		return 0.0;
	}

	match capacity_receiving_zone > part_of_people_flow {
		true => part_of_people_flow,
		false => capacity_receiving_zone,
	}
}

pub fn evac_def_modeling_step(bim: &Bim) {
	let area = bim.area();

	let average_size = area / bim.zones.len() as f64;
	let hxy = average_size.sqrt(); // характерный размер области, м
	unsafe {
		EVAC_MODELING_STEP_RUST = match EVAC_MODELING_STEP_RUST == 0.0 {
			true => hxy / EVAC_SPEED_MAX_RUST * 0.1,
			false => EVAC_MODELING_STEP_RUST, // Шаг моделирования, мин
		}
	}
}

pub fn evac_moving_step_test_with_log_rust(
	graph: &BimGraph,
	zones: &mut [BimZone],
	transits: &mut [BimTransit],
) {
	reset_zones(zones);
	reset_transits(transits);

	let mut zones_to_process: Vec<BimZone> = vec![];

	let outside_id = graph.head.len() - 1;
	let mut ptr = Some(Box::new(graph.head[outside_id].clone()));
	let mut receiving_zone_id = outside_id;

	for _ in 0..zones.len() {
		for _ in 0..zones[receiving_zone_id].outputs.len() {
			if let Some(ptr_box) = ptr {
				let transit = &mut transits[ptr_box.eid];

				if transit.is_visited || transit.is_blocked {
					ptr = ptr_box.next;
					continue;
				}

				let giving_zone_id = ptr_box.dest;

				zones[receiving_zone_id].potential =
					potential_element(&zones[receiving_zone_id], &zones[giving_zone_id], transit);

				let moved_people =
					part_people_flow(&zones[receiving_zone_id], &zones[giving_zone_id], transit);
				zones[receiving_zone_id].number_of_people += moved_people;
				zones[giving_zone_id].number_of_people -= moved_people;
				transit.no_proceeding = moved_people;

				zones[giving_zone_id].is_visited = true;
				transit.is_visited = true;

				if zones[giving_zone_id].outputs.len() > 1
					&& !zones[giving_zone_id].is_blocked
					&& !zones_to_process
						.iter()
						.any(|x| x.id == zones[giving_zone_id].id)
				{
					zones_to_process.push(zones[giving_zone_id].clone());
				}

				ptr = ptr_box.next;
			} else {
				break;
			}
		}

		zones_to_process.sort_by(|a, b| a.potential.total_cmp(&b.potential));

		if !zones_to_process.is_empty() {
			let deleted_zone = zones_to_process.remove(0);
			receiving_zone_id = zones
				.iter()
				.position(|zone| deleted_zone.uuid.eq(&zone.uuid))
				.unwrap_or_else(|| panic!("Zone not found!"));
			ptr = Some(Box::new(graph.head[deleted_zone.id as usize].clone()));
		}
	}
}

// pub fn evac_moving_step(
// 	graph: &UnGraph<&BimZone, &BimTransit>,
// 	zones: &mut [BimZone],
// 	transits: &mut [BimTransit],
// ) {
// 	reset_zones(zones);
// 	reset_transits(transits);
//
// 	let mut zones_to_process: Vec<BimZone> = vec![];
//
// 	let outside_id = graph.head.len() - 1;
// 	let mut ptr = Some(Box::new(graph.head[outside_id].clone()));
// 	let mut receiving_zone_id = outside_id;
//
// 	for _ in 0..zones.len() {
// 		for _ in 0..zones[receiving_zone_id].outputs.len() {
// 			if let Some(ptr_box) = ptr {
// 				let mut transit = &mut transits[ptr_box.eid];
//
// 				if transit.is_visited || transit.is_blocked {
// 					ptr = ptr_box.next;
// 					continue;
// 				}
//
// 				let giving_zone_id = ptr_box.dest;
//
// 				zones[receiving_zone_id].potential =
// 					potential_element(&zones[receiving_zone_id], &zones[giving_zone_id], transit);
//
// 				let moved_people =
// 					part_people_flow(&zones[receiving_zone_id], &zones[giving_zone_id], transit);
// 				zones[receiving_zone_id].number_of_people += moved_people;
// 				zones[giving_zone_id].number_of_people -= moved_people;
// 				transit.no_proceeding = moved_people;
//
// 				zones[giving_zone_id].is_visited = true;
// 				transit.is_visited = true;
//
// 				if zones[giving_zone_id].outputs.len() > 1
// 					&& !zones[giving_zone_id].is_blocked
// 					&& !zones_to_process
// 						.iter()
// 						.any(|x| x.id == zones[giving_zone_id].id)
// 				{
// 					zones_to_process.push(zones[giving_zone_id].clone());
// 				}
//
// 				ptr = ptr_box.next;
// 			} else {
// 				break;
// 			}
// 		}
//
// 		zones_to_process.sort_by(|a, b| a.potential.total_cmp(&b.potential));
//
// 		if !zones_to_process.is_empty() {
// 			let deleted_zone = zones_to_process.remove(0);
// 			receiving_zone_id = zones
// 				.iter()
// 				.position(|zone| deleted_zone.uuid.eq(&zone.uuid))
// 				.unwrap_or_else(|| panic!("Zone not found!"));
// 			ptr = Some(Box::new(graph.head[deleted_zone.id as usize].clone()));
// 		}
// 	}
// }

pub fn reset_zones(zones: &mut [BimZone]) {
	for zone in zones {
		zone.is_visited = false;
		zone.potential = match zone.sign {
			BimElementSign::Outside => 0.0,
			_ => f64::from(f32::MAX),
		};
	}
}

pub fn reset_transits(transits: &mut [BimTransit]) {
	for transit in transits {
		transit.is_visited = false;
		transit.no_proceeding = 0.0;
	}
}

pub fn set_speed_max(speed: f64) {
	unsafe {
		EVAC_SPEED_MAX_RUST = speed;
	}
}

pub fn set_density_min(density: f64) {
	unsafe {
		EVAC_DENSITY_MIN_RUST = density;
	}
}

pub fn set_density_max(density: f64) {
	unsafe {
		EVAC_DENSITY_MAX_RUST = density;
	}
}

pub fn set_modeling_step(step: f64) {
	unsafe {
		EVAC_MODELING_STEP_RUST = step;
	}
}

pub fn get_time_s() -> f64 {
	unsafe { EVAC_TIME_RUST * 60.0 }
}

pub fn get_time_m() -> f64 {
	unsafe { EVAC_TIME_RUST }
}

pub fn time_inc() {
	unsafe {
		EVAC_TIME_RUST += EVAC_MODELING_STEP_RUST;
	}
}

pub fn time_reset() {
	unsafe {
		EVAC_TIME_RUST = 0.0;
	}
}

#[cfg(test)]
mod tests {
	use super::super::bim_polygon_tools::Polygon;
	use super::super::json_object::Point;
	use super::*;
	use rstest::*;
	use uuid::uuid;

	#[fixture]
	fn receiving_zone() -> BimZone {
		BimZone {
			id: 1,
			name: "Receiving zone".to_string(),
			uuid: uuid!("00000000-0000-0000-0000-000000000000"),
			outputs: vec![uuid!("00000000-0000-0000-0000-000000000000")],
			area: 10.0,
			z_level: 1.0,
			number_of_people: 10.0,
			hazard_level: 0,
			is_blocked: false,
			is_visited: false,
			is_safe: true,
			sign: BimElementSign::Room,
			size_z: 2.0,
			polygon: Polygon::default(),
			potential: 1.0,
		}
	}

	#[fixture]
	fn transmitting_zone() -> BimZone {
		BimZone {
			id: 2,
			name: "Transmitting zone".to_string(),
			uuid: uuid!("00000000-0000-0000-0000-000000000000"),
			outputs: vec![uuid!("00000000-0000-0000-0000-000000000000")],
			area: 10.0,
			z_level: 1.0,
			number_of_people: 10.0,
			hazard_level: 0,
			is_blocked: false,
			is_visited: false,
			is_safe: true,
			sign: BimElementSign::Room,
			size_z: 2.0,
			polygon: Polygon::default(),
			potential: 1.0,
		}
	}

	#[fixture]
	fn transit() -> BimTransit {
		BimTransit {
			uuid: uuid!("00000000-0000-0000-0000-000000000000"),
			id: 1,
			name: "Transit".to_string(),
			outputs: vec![uuid!("00000000-0000-0000-0000-000000000000")],
			polygon: Polygon::default(),
			size_z: 2.0,
			z_level: 1.0,
			width: 1.0,
			no_proceeding: 0.0,
			sign: BimElementSign::DoorWay,
			is_visited: false,
			is_blocked: false,
		}
	}

	#[rstest]
	fn speed_in_element_eq(transmitting_zone: BimZone, receiving_zone: BimZone) {
		unsafe {
			EVAC_SPEED_MAX_RUST = 100.0;
		}

		assert_eq!(
			speed_in_element(&receiving_zone, &transmitting_zone),
			80.13633567871892
		);
	}

	#[rstest]
	fn speed_at_exit_eq(receiving_zone: BimZone, transmitting_zone: BimZone) {
		unsafe {
			EVAC_SPEED_MAX_RUST = 100.0;
		}
		let transit_width = 1.0;

		assert_eq!(
			speed_at_exit(&receiving_zone, &transmitting_zone, transit_width),
			80.13633567871892
		);
	}

	#[rstest]
	fn change_num_of_people_eq(transmitting_zone: BimZone) {
		unsafe {
			EVAC_MODELING_STEP_RUST = 0.01;
		}
		let transit_width = 1.0;
		let speed_at_exit = 50.0;

		assert_eq!(
			change_num_of_people(&transmitting_zone, transit_width, speed_at_exit),
			0.5
		);
	}

	#[rstest]
	fn potential_element_eq(
		receiving_zone: BimZone,
		transmitting_zone: BimZone,
		transit: BimTransit,
	) {
		assert_eq!(
			potential_element(&receiving_zone, &transmitting_zone, &transit),
			1.039461221097587
		);
	}

	#[rstest]
	fn part_people_flow_eq(
		receiving_zone: BimZone,
		transmitting_zone: BimZone,
		transit: BimTransit,
	) {
		unsafe {
			EVAC_DENSITY_MIN_RUST = 0.1;
			EVAC_DENSITY_MAX_RUST = 5.0;
		}

		assert_eq!(
			part_people_flow(&receiving_zone, &transmitting_zone, &transit),
			0.8013633567871893
		);
	}

	#[rstest]
	fn part_people_flow_eq_1() {
		let receiving_zone = BimZone {
			uuid: uuid!("00000000-0000-0000-0000-000000000000"),
			id: 6,
			name: String::from("Outside"),
			polygon: Polygon { points: vec![] },
			outputs: vec![
				uuid!("f2590703-5d39-43c7-8241-5e7dd246e3d4"),
				uuid!("003c0e9b-2671-4e4a-bc23-31456986725c"),
			],
			size_z: 1.7976931348623157e+308,
			z_level: 0.0,
			number_of_people: 0.0,
			potential: 0.05983352150182077,
			area: 1.7976931348623157e+308,
			hazard_level: 0,
			sign: BimElementSign::Outside,
			is_visited: false,
			is_blocked: false,
			is_safe: true,
		};

		let transmitting_zone = BimZone {
			uuid: uuid!("37a611a3-b0b4-491f-b3de-f0eb7a06349c"),
			id: 3,
			name: String::from("Comp 5 (00 : 6349c)"),
			polygon: Polygon {
				points: vec![
					Point {
						x: 8.049085116975721,
						y: 3.939957782245948,
					},
					Point {
						x: 13.925474407478942,
						y: 3.939957782245948,
					},
					Point {
						x: 13.925474407478942,
						y: 0.03282578402084191,
					},
					Point {
						x: 8.03339382381016,
						y: 0.03282578402084263,
					},
					Point {
						x: 8.049085116975721,
						y: 3.939957782245948,
					},
				],
			},
			outputs: vec![
				uuid!("b0611c37-febb-44ca-b8b4-f4ed4626d007"),
				uuid!("ad4962eb-dfd5-4295-8693-18066f5e82e3"),
				uuid!("003c0e9b-2671-4e4a-bc23-31456986725c"),
			],
			size_z: 3.0,
			z_level: 0.0,
			number_of_people: 22.990482607762807,
			potential: 1.7976931348623157e+308,
			area: 22.990482607762807,
			hazard_level: 0,
			sign: BimElementSign::Room,
			is_visited: false,
			is_blocked: false,
			is_safe: false,
		};

		let transit = BimTransit {
			uuid: uuid!("003c0e9b-2671-4e4a-bc23-31456986725c"),
			id: 6,
			name: String::from("Выход (00 : 6725c) 6349c<->"),
			outputs: vec![uuid!("37a611a3-b0b4-491f-b3de-f0eb7a06349c")],
			polygon: Polygon {
				points: vec![
					Point {
						x: 10.98923990097158,
						y: -0.23217549927336378,
					},
					Point {
						x: 10.98923990097158,
						y: 0.3678245007266362,
					},
					Point {
						x: 12.98923990097158,
						y: 0.3678245007266362,
					},
					Point {
						x: 12.98923990097158,
						y: -0.23217549927336378,
					},
					Point {
						x: 10.98923990097158,
						y: -0.23217549927336378,
					},
				],
			},
			size_z: 2.0,
			z_level: 0.0,
			width: 2.0,
			no_proceeding: 0.0,
			sign: BimElementSign::DoorWayOut,
			is_visited: false,
			is_blocked: false,
		};

		unsafe {
			EVAC_DENSITY_MIN_RUST = 0.1;
			EVAC_DENSITY_MAX_RUST = 5.0;
		}

		assert_eq!(
			part_people_flow(&receiving_zone, &transmitting_zone, &transit),
			1.6027267135743786
		);
	}
}
