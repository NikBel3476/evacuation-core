use super::json_object::Point;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use std::cmp::Ordering;
use triangle_rs::Delaunay;
use triangulate::PolygonList;
use triangulate::{formats, ListFormat};

pub struct Line {
	pub p1: Point,
	pub p2: Point,
}

#[derive(Debug, Clone, Default)]
pub struct Polygon {
	pub points: Vec<Point>,
	// delaunay: Delaunay,
}

impl Polygon {
	// triangulate
	pub fn triangulate(&self) -> Vec<[Point; 3]> {
		let mut triangulated_indices = Vec::<[usize; 2]>::new();
		// exclude last point because it is copy of first point
		let polygon = vec![self.points[0..self.points.len() - 1]
			.iter()
			.map(|point| [point.x, point.y])
			.collect::<Vec<[f64; 2]>>()];

		polygon
			.triangulate(
				formats::IndexedListFormat::new(&mut triangulated_indices).into_fan_format(),
			)
			.expect("Triangulation failed");
		triangulated_indices
			.chunks(3)
			.map(|point_indexes| {
				let point0 = polygon.get_vertex(point_indexes[0]);
				let point1 = polygon.get_vertex(point_indexes[1]);
				let point2 = polygon.get_vertex(point_indexes[2]);

				[
					Point {
						x: point0[0],
						y: point0[1],
					},
					Point {
						x: point1[0],
						y: point1[1],
					},
					Point {
						x: point2[0],
						y: point2[1],
					},
				]
			})
			.collect()
	}

	/// #Returns
	/// Массив номеров точек треугольников
	///
	/// https://userpages.umbc.edu/~rostamia/cbook/triangle.html
	// spade
	// pub fn triangulate(&self) -> ConstrainedDelaunayTriangulation<Point2<f64>> {
	// 	let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::new();
	// 	self.points[0..self.points.len() - 1]
	// 		.iter()
	// 		.enumerate()
	// 		.for_each(|(i, point)| {
	// 			let (vertex0, vertex1) = match i >= self.points.len() - 1 {
	// 				true => (
	// 					Point2::new(point.x, point.y),
	// 					Point2::new(self.points[0].x, self.points[0].y),
	// 				),
	// 				false => (
	// 					Point2::new(point.x, point.y),
	// 					Point2::new(self.points[i + 1].x, self.points[i + 1].y),
	// 				),
	// 			};
	//
	// 			// cdt.insert(vertex0).unwrap_or_else(|err| {
	// 			// 	panic!("{err}\n{:?}", vertex0);
	// 			// });
	// 			// cdt.insert(vertex1).unwrap_or_else(|err| {
	// 			// 	panic!("{err}\n{:?}", vertex1);
	// 			// });
	//
	// 			cdt.add_constraint_edge(vertex0, vertex1)
	// 				.unwrap_or_else(|err| {
	// 					panic!("{err}\n{:?}\n{:?}", vertex0, vertex1);
	// 				});
	// 		});
	//
	// 	cdt
	// }

	// triangle-rs
	// pub fn triangulate(&self) -> Delaunay {
	// 	let polygon = self
	// 		.points
	// 		.iter()
	// 		.flat_map(|point| vec![point.x, point.y])
	// 		.collect::<Vec<f64>>();
	//
	// 	let tri = triangle_rs::Builder::new()
	// 		.set_switches("pQ")
	// 		.add_polygon(&polygon)
	// 		.build();
	//
	// 	tri
	// }

	// triangulate
	pub fn area(&self) -> f64 {
		let triangles = self.triangulate();
		triangles.iter().fold(0.0, |total_area, triangle| {
			total_area + triangle_area(&triangle[0], &triangle[1], &triangle[2])
		})
	}

	// triangle-rs
	// pub fn area(&self) -> f64 {
	// 	let tri = self.triangulate();
	// 	tri.area()
	// }

	// spade
	// pub fn area(&self) -> f64 {
	// 	// Ok(self.delaunay.area())
	// 	let cdt = self.triangulate();
	// 	cdt.inner_faces()
	// 		.fold(0.0, |area, triangle| area + triangle.area())
	// }

	// triangulate
	pub fn is_point_inside(&self, point: &Point) -> Result<bool, String> {
		if self.points.len() < 3 {
			return Err(String::from("Less than 3 vertices"));
		}

		let triangles = self.triangulate();
		Ok(triangles.iter().any(|triangle| {
			is_point_inside_triangle(point, triangle)
			// is_point_in_triangle(
			// 	triangle[0].x,
			// 	triangle[0].y,
			// 	triangle[1].x,
			// 	triangle[1].y,
			// 	triangle[2].x,
			// 	triangle[2].y,
			// 	point.x,
			// 	point.y,
			// )
		}))
	}

	// triangle-rs
	// pub fn is_point_inside(&self, point: &Point) -> Result<bool, String> {
	// 	if self.points.len() < 3 {
	// 		return Err(String::from("Less than 3 vertices"));
	// 	}
	//
	// 	let tri = self.triangulate();
	// 	Ok(self.delaunay.is_point_inside(&[point.x, point.y]))
	// }

	// spade
	// pub fn is_point_inside(&self, point: &Point) -> Result<bool, String> {
	// 	if self.points.len() < 3 {
	// 		return Err(String::from("Less than 3 vertices"));
	// 	}
	//
	// 	let cdt = self.triangulate();
	// 	Ok(cdt.inner_faces().any(|face| {
	// 		let [a, b, c] = face.vertices();
	// 		let d1 = self.sign(point, &a.position(), &b.position());
	// 		let d2 = self.sign(point, &b.position(), &c.position());
	// 		let d3 = self.sign(point, &c.position(), &a.position());
	// 		let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
	// 		let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
	//
	// 		!(has_neg && has_pos)
	// 	}))
	// }

	fn sign(&self, p1: &Point, p2: &Point2<f64>, p3: &Point2<f64>) -> f64 {
		(p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
	}
}

impl From<&[Point]> for Polygon {
	fn from(points: &[Point]) -> Self {
		Self {
			points: points.to_vec(),
			// delaunay: triangulate(points),
		}
	}
}

fn triangulate(points: &[Point]) -> Delaunay {
	let polygon = points
		.iter()
		.flat_map(|point| vec![point.x, point.y])
		.collect::<Vec<f64>>();

	let tri = triangle_rs::Builder::new()
		.set_switches("pQ")
		.add_polygon(&polygon)
		.build();

	tri
}

fn where_point_rust(a_ax: f64, a_ay: f64, a_bx: f64, a_by: f64, a_px: f64, a_py: f64) -> i32 {
	let s = (a_bx - a_ax) * (a_py - a_ay) - (a_by - a_ay) * (a_px - a_ax);
	match s.total_cmp(&0.0) {
		Ordering::Greater => 1, // Точка слева от вектора AB
		Ordering::Less => -1,   // Точка справа от вектора AB
		Ordering::Equal => 0,   // Точка на векторе, прямо по вектору или сзади вектора
	}
}

pub fn is_point_inside_triangle(point: &Point, triangle: &[Point; 3]) -> bool {
	let d1 = sign(point, &triangle[0], &triangle[1]);
	let d2 = sign(point, &triangle[1], &triangle[2]);
	let d3 = sign(point, &triangle[2], &triangle[0]);
	let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
	let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

	!(has_neg && has_pos)
}

fn sign(p1: &Point, p2: &Point, p3: &Point) -> f64 {
	(p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

#[allow(clippy::too_many_arguments)]
fn is_point_in_triangle(
	a_ax: f64,
	a_ay: f64,
	a_bx: f64,
	a_by: f64,
	a_cx: f64,
	a_cy: f64,
	a_px: f64,
	a_py: f64,
) -> bool {
	let q1 = where_point_rust(a_ax, a_ay, a_bx, a_by, a_px, a_py);
	let q2 = where_point_rust(a_bx, a_by, a_cx, a_cy, a_px, a_py);
	let q3 = where_point_rust(a_cx, a_cy, a_ax, a_ay, a_px, a_py);

	q1 >= 0 && q2 >= 0 && q3 >= 0
}

// pub fn is_point_in_polygon(point: &Point, polygon: &Polygon) -> Result<bool, String> {
// 	if polygon.points.len() < 3 {
// 		return Err(String::from("Less than 3 vertices"));
// 	}
//
// 	let tri = polygon.triangulate();
// 	Ok(tri.is_point_inside(&[point.x, point.y]))
// }

pub fn triangle_area(p1: &Point, p2: &Point, p3: &Point) -> f64 {
	0.5 * ((p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)).abs()
}

/// signed area of a triangle
pub fn area(p1: &Point, p2: &Point, p3: &Point) -> f64 {
	(p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x)
}

/// https://e-maxx.ru/algo/segments_intersection_checking
pub fn intersect_1_rust(mut a: f64, mut b: f64, mut c: f64, mut d: f64) -> bool {
	if a > b {
		std::mem::swap(&mut a, &mut b);
	}
	if c > d {
		std::mem::swap(&mut c, &mut d);
	}

	a.max(c) <= b.min(d)
}

/// check if two segments intersect
pub fn is_intersect_line(l1: &Line, l2: &Line) -> bool {
	let p1 = &l1.p1;
	let p2 = &l1.p2;
	let p3 = &l2.p1;
	let p4 = &l2.p2;

	intersect_1_rust(p1.x, p2.x, p3.x, p4.x)
		&& intersect_1_rust(p1.y, p2.y, p3.y, p4.y)
		&& area(p1, p2, p3) * area(p1, p2, p4) <= 0.0
		&& area(p3, p4, p1) * area(p3, p4, p2) <= 0.0
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::*;

	#[fixture]
	fn triangle_polygon_with_area_1() -> Polygon {
		Polygon {
			points: vec![
				Point { x: 0.0, y: -1.0 },
				Point { x: 1.0, y: 0.0 },
				Point { x: 0.0, y: 1.0 },
				Point { x: 0.0, y: -1.0 },
			],
		}
	}

	#[fixture]
	fn triangle_polygon_with_area_0_5() -> Polygon {
		Polygon {
			points: vec![
				Point { x: 0.0, y: 0.0 },
				Point { x: 1.0, y: 0.0 },
				Point { x: 0.0, y: 1.0 },
				Point { x: 0.0, y: 0.0 },
			],
		}
	}

	#[fixture]
	fn parallelogram_polygon() -> Polygon {
		Polygon {
			points: vec![
				Point { x: -2.0, y: -1.0 },
				Point { x: 2.0, y: -1.0 },
				Point { x: 3.0, y: 1.0 },
				Point { x: -1.0, y: 1.0 },
				Point { x: -2.0, y: -1.0 },
			],
		}
	}

	#[fixture]
	fn complex_figure_with_right_angles_polygon() -> Polygon {
		Polygon {
			points: vec![
				Point {
					x: 35.97872543334961,
					y: -34.659114837646484,
				},
				Point {
					x: 35.97872543334961,
					y: -37.01911163330078,
				},
				Point {
					x: 33.9708251953125,
					y: -37.01911163330078,
				},
				Point {
					x: 33.9708251953125,
					y: -37.219112396240234,
				},
				Point {
					x: 34.07872772216797,
					y: -37.219112396240234,
				},
				Point {
					x: 34.0787277221679,
					y: -38.4352912902832,
				},
				Point {
					x: 33.15372467041016,
					y: -38.4352912902832,
				},
				Point {
					x: 33.153724670410156,
					y: -37.219112396240234,
				},
				Point {
					x: 33.25210189819336,
					y: -37.219112396240234,
				},
				Point {
					x: 33.25210189819336,
					y: -37.01911163330078,
				},
				Point {
					x: 32.90689468383789,
					y: -37.01911163330078,
				},
				Point {
					x: 32.90689468383789,
					y: -37.219112396240234,
				},
				Point {
					x: 33.003726959228516,
					y: -37.219112396240234,
				},
				Point {
					x: 33.00372695922856,
					y: -38.4352912902832,
				},
				Point {
					x: 32.0787277221679,
					y: -38.4352912902832,
				},
				Point {
					x: 32.07872772216797,
					y: -37.219112396240234,
				},
				Point {
					x: 32.193763732910156,
					y: -37.219112396240234,
				},
				Point {
					x: 32.19376373291015,
					y: -37.01911163330078,
				},
				Point {
					x: 30.50872802734375,
					y: -37.01911163330078,
				},
				Point {
					x: 30.50872802734375,
					y: -34.659114837646484,
				},
				Point {
					x: 35.97872543334961,
					y: -34.659114837646484,
				},
			],
		}
	}

	#[fixture]
	fn square_polygon() -> Polygon {
		Polygon {
			points: vec![
				Point { x: 0.0, y: 0.0 },
				Point { x: 1.0, y: 0.0 },
				Point { x: 1.0, y: 1.0 },
				Point { x: 0.0, y: 1.0 },
				Point { x: 0.0, y: 0.0 },
			],
		}
	}

	#[fixture]
	fn rectangle_for_intersection_test() -> Polygon {
		Polygon {
			points: vec![
				Point {
					x: 1.0804720876503242,
					y: 9.784116583159095,
				},
				Point {
					x: 9.452596550210751,
					y: 9.830117267019318,
				},
				Point {
					x: 9.475596892140864,
					y: 1.2969904109481103,
				},
				Point {
					x: 1.1034724295804352,
					y: 1.2969904109481103,
				},
				Point {
					x: 1.0804720876503242,
					y: 9.784116583159095,
				},
			],
		}
	}

	#[fixture]
	fn points_outside_rectangle_for_intersection_test() -> Vec<Point> {
		vec![
			Point {
				x: 7.198563041059868,
				y: 10.888132995804426,
			},
			Point {
				x: 8.877588001957976,
				y: 10.934133679664647,
			},
		]
	}

	#[fixture]
	fn points_inside_rectangle_for_intersection_test() -> Vec<Point> {
		vec![
			Point {
				x: 8.854587660027866,
				y: 9.577113505788097,
			},
			Point {
				x: 7.198563041059868,
				y: 9.554113163857984,
			},
		]
	}

	#[fixture]
	fn points_outside_figure_for_intersection_test() -> Vec<Point> {
		vec![
			Point {
				x: 31.87872886657715,
				y: -38.24702072143555,
			},
			Point {
				x: 31.87872886657715,
				y: -37.34701919555664,
			},
		]
	}

	#[fixture]
	fn points_inside_figure_for_intersection_test() -> Vec<Point> {
		vec![
			Point {
				x: 32.07872772216797,
				y: -38.24702072143555,
			},
			Point {
				x: 32.07872772216797,
				y: -37.34701919555664,
			},
		]
	}

	#[rstest]
	#[case::triangle_polygon(triangle_polygon_with_area_1(), 1.0)]
	#[case::parallelogram_polygon(parallelogram_polygon(), 8.0)]
	#[case::parallelogram_polygon(complex_figure_with_right_angles_polygon(), 15.44548203003071)]
	fn figure_area(#[case] polygon: Polygon, #[case] expected_area: f64) {
		assert_eq!(polygon.area(), expected_area)
	}

	#[rstest]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 0.0, y: 0.0 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 0.5, y: 0.0 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 1.0, y: 0.0 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 0.5, y: 0.5 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 0.0, y: 1.0 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 0.0, y: 0.5 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 0.0, y: 0.0 })]
	fn point_inside_triangle(#[case] triangle_polygon: Polygon, #[case] point: Point) {
		assert!(
			triangle_polygon.is_point_inside(&point).unwrap(),
			"\nPoint {:?} should be inside triangle\n",
			point
		)
	}

	#[rstest]
	#[case(triangle_polygon_with_area_0_5(), Point { x: -1.0, y: -1.0 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 0.5, y: -1.0 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 1.5, y: -0.5 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: 1.0, y: 1.0 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: -0.5, y: 1.5 })]
	#[case(triangle_polygon_with_area_0_5(), Point { x: -0.5, y: 0.5 })]
	fn point_outside_triangle(#[case] triangle_polygon: Polygon, #[case] point: Point) {
		assert!(
			!triangle_polygon.is_point_inside(&point).unwrap(),
			"\nPoint {:?} should be outside triangle\n",
			point
		)
	}

	#[rstest]
	#[case(square_polygon(), Point { x: 0.0, y: 0.0 })]
	#[case(square_polygon(), Point { x: 0.5, y: 0.0 })]
	#[case(square_polygon(), Point { x: 1.0, y: 0.0 })]
	#[case(square_polygon(), Point { x: 1.0, y: 0.5 })]
	#[case(square_polygon(), Point { x: 1.0, y: 1.0 })]
	#[case(square_polygon(), Point { x: 0.5, y: 1.0 })]
	#[case(square_polygon(), Point { x: 0.0, y: 1.0 })]
	#[case(square_polygon(), Point { x: 0.0, y: 0.5 })]
	#[case(square_polygon(), Point { x: 0.5, y: 0.5 })]
	fn point_inside_square(#[case] square_polygon: Polygon, #[case] point: Point) {
		assert!(
			square_polygon.is_point_inside(&point).unwrap(),
			"\nPoint {:?} should be inside square\n",
			point
		)
	}

	#[rstest]
	#[case(square_polygon(), Point { x: -0.5, y: -0.5 })]
	#[case(square_polygon(), Point { x: 0.5, y: -0.5 })]
	#[case(square_polygon(), Point { x: 1.5, y: -0.5 })]
	#[case(square_polygon(), Point { x: 1.5, y: 0.5 })]
	#[case(square_polygon(), Point { x: 1.5, y: 1.5 })]
	#[case(square_polygon(), Point { x: 0.5, y: 1.5 })]
	#[case(square_polygon(), Point { x: -0.5, y: 1.5 })]
	#[case(square_polygon(), Point { x: -0.5, y: 0.5 })]
	fn point_outside_square(#[case] square_polygon: Polygon, #[case] point: Point) {
		assert!(
			!square_polygon.is_point_inside(&point).unwrap(),
			"\nPoint {:?} should be outside square\n",
			point
		)
	}

	#[rstest]
	#[case::rectangles(
		rectangle_for_intersection_test(),
		points_outside_rectangle_for_intersection_test(),
		points_inside_rectangle_for_intersection_test()
	)]
	#[case::figure_and_a_rectangle(
		complex_figure_with_right_angles_polygon(),
		points_outside_figure_for_intersection_test(),
		points_inside_figure_for_intersection_test()
	)]
	fn polygon_intersection(
		#[case] polygon: Polygon,
		#[case] points_outside: Vec<Point>,
		#[case] points_inside: Vec<Point>,
	) {
		points_outside.iter().for_each(|point| {
			assert!(
				!polygon.is_point_inside(point).unwrap(),
				"\nPoint {:?} should be outside polygon\n",
				point
			)
		});

		points_inside.iter().for_each(|point| {
			assert!(
				polygon.is_point_inside(point).unwrap(),
				"\nPoint {:?} should be inside polygon\n",
				point
			)
		});
	}
}
