use std::collections::HashMap;
use std::time::{Duration, Instant};

use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, Image, Mesh, Rect, Text};
use ggez::input::keyboard::KeyCode;
use ggez::{glam::*, Context, GameError, GameResult};

type ScreenCoords = Vec2;
type ScreenDimensions = Vec2;

mod pin_point {
	/// Assume you want to draw something on a point of the canvas.
	/// But wait, the point you give to the function that does the drawing, is it supposed to
	/// be where the top left of the thing being rendered sould end up?
	/// It might be what you want, but maybe you find it easier in some cases to give the point
	/// on which the center of the thing should end up, or its bottom right corner, or the middle
	/// of its right side, etc.
	/// Well, this is exactly what a `PinPoint` allows to represent!
	/// If you pass `PinPoint::CENTER_LEFT` along a destination point to the hypothetical function
	/// that does the rendering, then the center left point of the thing being rendered will end up
	/// on the destination point you gave.
	#[derive(Clone, Copy)]
	pub struct PinPoint {
		x: f32,
		y: f32,
	}

	#[allow(dead_code)] // I don't want to comment out exactly the ones that happen to not be used!
	impl PinPoint {
		pub const TOP_LEFT: Self = PinPoint { x: 0.0, y: 0.0 };
		pub const TOP_CENTER: Self = PinPoint { x: 0.5, y: 0.0 };
		pub const TOP_RIGHT: Self = PinPoint { x: 1.0, y: 0.0 };
		pub const CENTER_LEFT: Self = PinPoint { x: 0.0, y: 0.5 };
		pub const CENTER_CENTER: Self = PinPoint { x: 0.5, y: 0.5 };
		pub const CENTER_RIGHT: Self = PinPoint { x: 1.0, y: 0.5 };
		pub const BOTTOM_LEFT: Self = PinPoint { x: 0.0, y: 1.0 };
		pub const BOTTOM_CENTER: Self = PinPoint { x: 0.5, y: 1.0 };
		pub const BOTTOM_RIGHT: Self = PinPoint { x: 1.0, y: 1.0 };
	}

	use ggez::graphics::Rect;

	use crate::{ScreenCoords, ScreenDimensions};

	impl PinPoint {
		/// When drawing something of the given dimensions to the given destination point
		/// that should be interpreted thanks to the given pin point, then the coords of where
		/// the top left corner of the thing being drawn end up is what this returns.
		pub fn actual_top_left_coords(
			self,
			dst: ScreenCoords,
			dims: ScreenDimensions,
		) -> ScreenCoords {
			(dst.x - (dims.x * self.x), dst.y - (dims.y * self.y)).into()
		}

		pub fn where_in_rect(self, dst_rect: Rect) -> ScreenCoords {
			ScreenCoords::new(
				dst_rect.x + dst_rect.w * self.x,
				dst_rect.y + dst_rect.h * self.y,
			)
		}
	}
}
use pin_point::PinPoint;

#[derive(Clone)]
struct VisElemPos {
	/// Mode depth means closer to the background (covered by what is closer to the foreground).
	depth: u32,
	/// What point in the *element being drawn* is supposed to be drawn at `coords`?
	pin_point: PinPoint,
	coords: ScreenCoords,
	parent: Option<Id>,
	/// What point in the *parent element* is `coords` supposed to be?
	in_parent_pin_point: PinPoint,
}

enum VisElemWhat {
	PinkRect,
	GreenDot,
}

impl VisElemWhat {
	fn dimensions(&self) -> ScreenDimensions {
		match self {
			VisElemWhat::PinkRect => (60.0, 100.0).into(),
			VisElemWhat::GreenDot => (20.0, 20.0).into(),
		}
	}
}

enum Animation {
	MoveTo {
		dst_pos: VisElemPos,
		/// In pixels per second.
		speed: f32,
	},
}

struct VisElem {
	pos: VisElemPos,
	animations: Vec<Animation>,
	what: VisElemWhat,
}

mod id_generator {
	pub struct IdGenerator {
		next_id_value: u64,
	}

	impl IdGenerator {
		pub fn new() -> IdGenerator {
			IdGenerator { next_id_value: 0 }
		}

		pub fn generate_id(&mut self) -> Id {
			let id_value = self.next_id_value;
			self.next_id_value += 1;
			Id(id_value)
		}
	}

	#[derive(Clone, Copy, PartialEq, Eq, Hash)]
	pub struct Id(u64);
}

use id_generator::{Id, IdGenerator};

struct Game {
	id_generator: IdGenerator,
	context_size: (f32, f32),
	vis_elems: HashMap<Id, VisElem>,
	test_rect_ids: Vec<Id>,
	test_dot_id: Id,
}

impl Game {
	fn new(ctx: &Context) -> Result<Game, GameError> {
		let mut id_generator = IdGenerator::new();
		let mut vis_elems = HashMap::new();

		let mut test_rect_ids = vec![];
		for i in 0..3 {
			let test_rect_id = id_generator.generate_id();
			test_rect_ids.push(test_rect_id);
			vis_elems.insert(
				test_rect_id,
				VisElem {
					animations: vec![],
					pos: VisElemPos {
						depth: 1000,
						pin_point: PinPoint::CENTER_CENTER,
						coords: ScreenCoords::new(-300.0 + 300.0 * i as f32, 0.0),
						parent: None,
						in_parent_pin_point: PinPoint::CENTER_CENTER,
					},
					what: VisElemWhat::PinkRect,
				},
			);
		}

		let test_dot_id = id_generator.generate_id();
		vis_elems.insert(
			test_dot_id,
			VisElem {
				animations: vec![],
				pos: VisElemPos {
					depth: 500,
					pin_point: PinPoint::CENTER_CENTER,
					coords: ScreenCoords::new(0.0, -200.0),
					parent: None,
					in_parent_pin_point: PinPoint::CENTER_CENTER,
				},
				what: VisElemWhat::GreenDot,
			},
		);

		Ok(Game {
			id_generator,
			context_size: ctx.gfx.drawable_size(),
			vis_elems,
			test_rect_ids,
			test_dot_id,
		})
	}

	fn vis_elem_actual_rect(&self, pos: VisElemPos, dims: ScreenDimensions) -> Rect {
		let parent_rect = if let Some(parent_id) = pos.parent {
			let parent = self.vis_elems.get(&parent_id).unwrap();
			self.vis_elem_actual_rect(parent.pos.clone(), parent.what.dimensions())
		} else {
			Rect::new(0.0, 0.0, self.context_size.0, self.context_size.1)
		};
		let in_parent_coords = pos.in_parent_pin_point.where_in_rect(parent_rect);
		let self_coords = pos.pin_point.actual_top_left_coords(pos.coords, dims);
		let coords = in_parent_coords + self_coords;
		Rect::new(coords.x, coords.y, dims.x, dims.y)
	}

	fn draw_vis_elem(&self, ctx: &Context, canvas: &mut Canvas, id: Id) -> GameResult {
		let vis_elem = self.vis_elems.get(&id).unwrap();
		let rect = self.vis_elem_actual_rect(vis_elem.pos.clone(), vis_elem.what.dimensions());

		match vis_elem.what {
			VisElemWhat::PinkRect => {
				let rectangle = Mesh::new_rectangle(
					ctx,
					DrawMode::stroke(10.0),
					rect,
					Color::from_rgb(255, 100, 200),
				)?;
				canvas.draw(&rectangle, Vec2::new(0.0, 0.0));
			},
			VisElemWhat::GreenDot => {
				let circle = Mesh::new_circle(
					ctx,
					DrawMode::fill(),
					rect.center(),
					rect.w,
					0.1,
					Color::from_rgb(100, 255, 200),
				)?;
				canvas.draw(&circle, Vec2::new(0.0, 0.0));
			},
		}

		Ok(())
	}
}

impl ggez::event::EventHandler<ggez::GameError> for Game {
	fn key_down_event(
		&mut self,
		ctx: &mut Context,
		input: ggez::input::keyboard::KeyInput,
		_repeated: bool,
	) -> GameResult {
		if let Some(keycode) = input.keycode {
			match keycode {
				KeyCode::Space => {
					self
						.vis_elems
						.get_mut(&self.test_dot_id)
						.unwrap()
						.animations
						.push(Animation::MoveTo {
							dst_pos: VisElemPos {
								depth: 500,
								pin_point: PinPoint::CENTER_CENTER,
								coords: Vec2::new(0.0, 0.0),
								parent: Some(self.test_rect_ids[0]),
								in_parent_pin_point: PinPoint::TOP_RIGHT,
							},
							speed: 1000.0,
						});
				},
				KeyCode::Escape => ctx.request_quit(),
				_ => {},
			}
		}
		Ok(())
	}

	fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) -> GameResult {
		self.context_size = ctx.gfx.drawable_size();
		Ok(())
	}

	fn update(&mut self, ctx: &mut Context) -> GameResult {
		let dt_in_seconds = ctx.time.delta().as_secs_f32();

		let ids: Vec<_> = self.vis_elems.keys().copied().collect();
		for id in ids.into_iter() {
			let mut vis_elem = self.vis_elems.remove(&id).unwrap();
			let mut finished_animation_indices = vec![];

			for (animation_i, animation) in vis_elem.animations.iter().enumerate() {
				match animation {
					Animation::MoveTo { dst_pos, speed } => {
						let dims = vis_elem.what.dimensions();
						let dst_rect = self.vis_elem_actual_rect(dst_pos.clone(), dims);
						let cur_rect = self.vis_elem_actual_rect(vis_elem.pos.clone(), dims);
						let dst_top_left: Vec2 = dst_rect.point().into();
						let cur_top_left: Vec2 = cur_rect.point().into();
						let motion: Vec2 =
							(dst_top_left - cur_top_left).normalize_or_zero() * (*speed * dt_in_seconds);

						if dst_top_left == cur_top_left
							|| (cur_top_left + motion - dst_top_left).length()
								>= (dst_top_left - cur_top_left).length()
						{
							vis_elem.pos = dst_pos.clone();
							finished_animation_indices.push(animation_i);
						} else {
							vis_elem.pos = dst_pos.clone();
							vis_elem.pos.coords = cur_top_left - dst_top_left + motion;
						}
					},
				}
			}

			for animation_i in finished_animation_indices.into_iter().rev() {
				vis_elem.animations.remove(animation_i);
			}

			self.vis_elems.insert(id, vis_elem);
		}

		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

		// Draw all visual elements,
		// iterating over the set of their ids without having `self` borrowed,
		// and deeper elements first.
		let mut ids: Vec<_> = self.vis_elems.keys().copied().collect();
		ids.sort_unstable_by_key(|id| self.vis_elems.get(id).unwrap().pos.depth);
		for id in ids.into_iter().rev() {
			self.draw_vis_elem(ctx, &mut canvas, id)?;
		}

		let fps = ctx.time.fps().round() as i64;
		canvas.draw(
			Text::new(format!("FPS: {fps}")).set_scale(26.0),
			DrawParam::from(Vec2::new(0.0, 0.0)).color(Color::WHITE),
		);

		canvas.finish(ctx)?;
		Ok(())
	}
}

fn main() -> GameResult {
	let (ctx, event_loop) = ggez::ContextBuilder::new("frog_dream", "Anima")
		.window_setup(
			ggez::conf::WindowSetup::default()
				.title("Frog Dream")
				.vsync(true),
		)
		.window_mode(
			ggez::conf::WindowMode::default()
				.resizable(true)
				.dimensions(1200.0, 900.0),
		)
		.build()?;
	let game = Game::new(&ctx)?;
	// Lets gooooooo!! Frog Dream!!! Yaaay ^^
	ggez::event::run(ctx, event_loop, game)
}
