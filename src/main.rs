use std::collections::HashMap;

use ggez::event::MouseButton;
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
	Card { card: Card, where_in_hand: Option<usize> },
	BattlefieldInsertPos(BattlefieldInsertPos),
}

impl VisElemWhat {
	fn dimensions(&self) -> ScreenDimensions {
		match self {
			VisElemWhat::Card { .. } => (200.0, 300.0).into(),
			VisElemWhat::BattlefieldInsertPos(_) => (40.0, 50.0).into(),
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
	selectable: bool,
	targetable: bool,
}

impl VisElem {
	fn move_to(&mut self, dst_pos: VisElemPos, speed: f32) {
		self
			.animations
			.retain(|animation| !matches!(animation, Animation::MoveTo { .. }));
		self.animations.push(Animation::MoveTo { dst_pos, speed });
	}
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

#[derive(Clone)]
struct Entity {
	test_color: Color,
}

#[derive(Clone)]
struct EntityOnBattlefield {
	entity: Entity,
	vis_elem_id: Id,
}

#[derive(Clone)]
enum Card {
	Entity(Entity),
}

#[derive(Clone)]
struct CardInHand {
	card: Card,
	vis_elem_id: Id,
}

struct Battlefield {
	friends: Vec<EntityOnBattlefield>,
	foes: Vec<EntityOnBattlefield>,
	friend_insert_pos_vis_elem_ids: Vec<Id>,
}

/// The position of something that is on the battlefield.
#[derive(Clone, Copy)]
enum BattlefieldPos {
	Friend(usize),
	Foe(usize),
}

/// Insertion position in the battlefield
/// (in the front or back of a team or between two members of a team).
///
/// The position in it is the position that the thing being inserted will have upon insertion.
#[derive(Clone, Copy)]
struct BattlefieldInsertPos(BattlefieldPos);

struct Game {
	id_generator: IdGenerator,
	context_size: (f32, f32),
	vis_elems: HashMap<Id, VisElem>,
	hand: Vec<CardInHand>,
	battlefield: Battlefield,
	cursor_pos: Option<Vec2>,
	hovered_vis_elem_id: Option<Id>,
	selected_vis_elem_id: Option<Id>,
}

impl Game {
	fn new(ctx: &Context) -> Result<Game, GameError> {
		let mut id_generator = IdGenerator::new();
		let mut vis_elems = HashMap::new();

		let mut hand = vec![];
		for i in 0..3 {
			let card_id = id_generator.generate_id();
			let card = Card::Entity(Entity { test_color: Color::from_rgb(0, 255, 255) });
			hand.push(CardInHand { card: card.clone(), vis_elem_id: card_id });
			vis_elems.insert(
				card_id,
				VisElem {
					animations: vec![],
					pos: VisElemPos {
						depth: 1000,
						pin_point: PinPoint::TOP_CENTER,
						coords: ScreenCoords::new(0.0, 0.0),
						parent: None,
						in_parent_pin_point: PinPoint::BOTTOM_CENTER,
					},
					what: VisElemWhat::Card { card, where_in_hand: Some(i) },
					selectable: true,
					targetable: false,
				},
			);
		}

		let mut game = Game {
			id_generator,
			context_size: ctx.gfx.drawable_size(),
			vis_elems,
			hand,
			battlefield: Battlefield {
				friends: vec![],
				foes: vec![],
				friend_insert_pos_vis_elem_ids: vec![],
			},
			cursor_pos: None,
			hovered_vis_elem_id: None,
			selected_vis_elem_id: None,
		};

		game.spawn_entity_on_battlefield(
			Entity { test_color: Color::from_rgb(255, 255, 150) },
			BattlefieldInsertPos(BattlefieldPos::Friend(0)),
		);
		game.spawn_entity_on_battlefield(
			Entity { test_color: Color::from_rgb(80, 255, 150) },
			BattlefieldInsertPos(BattlefieldPos::Foe(0)),
		);
		game.update_pos_of_entities_in_battlefield();

		game.update_pos_of_cards_in_hand();

		Ok(game)
	}

	fn spawn_entity_on_battlefield(&mut self, entity: Entity, insert_pos: BattlefieldInsertPos) {
		let vis_elem = VisElem {
			pos: VisElemPos {
				depth: 1000,
				pin_point: PinPoint::BOTTOM_CENTER,
				coords: ScreenCoords::new(0.0, 0.0),
				parent: None,
				in_parent_pin_point: PinPoint::TOP_CENTER,
			},
			animations: vec![],
			what: VisElemWhat::Card { card: Card::Entity(entity.clone()), where_in_hand: None },
			selectable: false,
			targetable: false,
		};
		let vis_elem_id = self.id_generator.generate_id();
		self.vis_elems.insert(vis_elem_id, vis_elem);
		let entity_on_battlefield = EntityOnBattlefield { entity, vis_elem_id };
		match insert_pos {
			BattlefieldInsertPos(BattlefieldPos::Friend(friend_i)) => {
				self
					.battlefield
					.friends
					.insert(friend_i, entity_on_battlefield);
			},
			BattlefieldInsertPos(BattlefieldPos::Foe(foe_i)) => {
				self.battlefield.foes.insert(foe_i, entity_on_battlefield);
			},
		}
	}

	fn update_pos_of_cards_in_hand(&mut self) {
		for hand_i in 0..self.hand.len() {
			let x = -((self.hand.len() as f32 - 1.0) * 220.0) / 2.0 + hand_i as f32 * 220.0;
			let vis_elem = self
				.vis_elems
				.get_mut(&self.hand[hand_i].vis_elem_id)
				.unwrap();
			vis_elem.move_to(
				VisElemPos {
					depth: 1000,
					pin_point: PinPoint::CENTER_CENTER,
					coords: Vec2::new(x, 225.0),
					parent: None,
					in_parent_pin_point: PinPoint::CENTER_CENTER,
				},
				1500.0,
			);
			match vis_elem.what {
				VisElemWhat::Card { ref mut where_in_hand, .. } => {
					*where_in_hand = Some(hand_i);
				},
				_ => panic!(),
			}
		}
	}

	fn update_pos_of_entities_in_battlefield(&mut self) {
		let len = (self.battlefield.friends.len() + self.battlefield.foes.len()) as f32;
		for friend_i in 0..self.battlefield.friends.len() {
			let x = -((len - 1.0) * 220.0 + 60.0) / 2.0 + friend_i as f32 * 220.0;
			let vis_elem = self
				.vis_elems
				.get_mut(&self.battlefield.friends[friend_i].vis_elem_id)
				.unwrap();
			vis_elem.move_to(
				VisElemPos {
					depth: 1000,
					pin_point: PinPoint::CENTER_CENTER,
					coords: Vec2::new(x, -225.0),
					parent: None,
					in_parent_pin_point: PinPoint::CENTER_CENTER,
				},
				1500.0,
			);
		}
		for foe_i in 0..self.battlefield.foes.len() {
			let x = -((len - 1.0) * 220.0 + 60.0) / 2.0
				+ 60.0 + (self.battlefield.friends.len() + foe_i) as f32 * 220.0;
			let vis_elem = self
				.vis_elems
				.get_mut(&self.battlefield.foes[foe_i].vis_elem_id)
				.unwrap();
			vis_elem.move_to(
				VisElemPos {
					depth: 1000,
					pin_point: PinPoint::CENTER_CENTER,
					coords: Vec2::new(x, -225.0),
					parent: None,
					in_parent_pin_point: PinPoint::CENTER_CENTER,
				},
				1500.0,
			);
		}

		for id in self.battlefield.friend_insert_pos_vis_elem_ids.iter() {
			self.vis_elems.remove(id);
		}
		self.battlefield.friend_insert_pos_vis_elem_ids.clear();
		for friend_insert_pos_i in 0..(self.battlefield.friends.len() + 1) {
			let id = self.id_generator.generate_id();
			self.battlefield.friend_insert_pos_vis_elem_ids.push(id);
			let x = -((len - 1.0) * 220.0 + 60.0) / 2.0 + (friend_insert_pos_i as f32 - 0.5) * 220.0;
			self.vis_elems.insert(
				id,
				VisElem {
					pos: VisElemPos {
						depth: 900,
						pin_point: PinPoint::CENTER_CENTER,
						coords: Vec2::new(x, -40.0),
						parent: None,
						in_parent_pin_point: PinPoint::CENTER_CENTER,
					},
					animations: vec![],
					what: VisElemWhat::BattlefieldInsertPos(BattlefieldInsertPos(
						BattlefieldPos::Friend(friend_insert_pos_i),
					)),
					selectable: false,
					targetable: false,
				},
			);
		}
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
		let hovered = self.hovered_vis_elem_id == Some(id);
		let selected = self.selected_vis_elem_id == Some(id);

		match vis_elem.what {
			VisElemWhat::Card { card: Card::Entity(Entity { test_color }), .. } => {
				let color = if selected {
					Color::from_rgb(255, 150, 180)
				} else if hovered {
					Color::YELLOW
				} else {
					test_color
				};
				let rectangle = Mesh::new_rectangle(ctx, DrawMode::stroke(10.0), rect, color)?;
				canvas.draw(&rectangle, Vec2::new(0.0, 0.0));
			},
			VisElemWhat::BattlefieldInsertPos(_) => {
				if vis_elem.targetable {
					let triangle = Mesh::new_polyline(
						ctx,
						DrawMode::stroke(3.0),
						&[
							Vec2::new(rect.center().x, rect.top()),
							Vec2::new(rect.left(), rect.bottom()),
							Vec2::new(rect.right(), rect.bottom()),
							Vec2::new(rect.center().x, rect.top()),
						],
						if hovered {
							Color::from_rgb(180, 255, 0)
						} else {
							Color::CYAN
						},
					)?;
					canvas.draw(&triangle, Vec2::new(0.0, 0.0));
				}
			},
		}

		Ok(())
	}
}

impl ggez::event::EventHandler<ggez::GameError> for Game {
	fn mouse_motion_event(
		&mut self,
		_ctx: &mut Context,
		x: f32,
		y: f32,
		_dx: f32,
		_dy: f32,
	) -> GameResult {
		self.cursor_pos = Some(Vec2::new(x, y));

		self.hovered_vis_elem_id = None;
		for (id, vis_elem) in self.vis_elems.iter() {
			let rect = self.vis_elem_actual_rect(vis_elem.pos.clone(), vis_elem.what.dimensions());
			if rect.contains(self.cursor_pos.unwrap()) {
				self.hovered_vis_elem_id = Some(*id);
			}
		}

		Ok(())
	}

	fn mouse_button_down_event(
		&mut self,
		_ctx: &mut Context,
		button: MouseButton,
		_x: f32,
		_y: f32,
	) -> GameResult {
		if let MouseButton::Left = button {
			if let Some(hovered_vis_elem_id) = self.hovered_vis_elem_id {
				let hovered_vis_elem = self.vis_elems.get(&hovered_vis_elem_id).unwrap();
				if hovered_vis_elem.selectable {
					self.selected_vis_elem_id = self.hovered_vis_elem_id;

					for insert_pos_vis_elem_id in self.battlefield.friend_insert_pos_vis_elem_ids.iter()
					{
						let insert_pos_vis_elem = self.vis_elems.get_mut(insert_pos_vis_elem_id).unwrap();
						insert_pos_vis_elem.targetable = true;
					}
				}
			}
		}
		Ok(())
	}

	fn mouse_button_up_event(
		&mut self,
		_ctx: &mut Context,
		button: MouseButton,
		_x: f32,
		_y: f32,
	) -> GameResult {
		if let MouseButton::Left = button {
			if let (Some(selected_vis_elem_id), Some(hovered_vis_elem_id)) =
				(self.selected_vis_elem_id, self.hovered_vis_elem_id)
			{
				let selected_card_where_in_hand =
					match self.vis_elems.get(&selected_vis_elem_id).unwrap().what {
						VisElemWhat::Card { where_in_hand, .. } => where_in_hand,
						_ => None,
					};
				let targeted_insert_pos = match self.vis_elems.get(&hovered_vis_elem_id).unwrap().what {
					VisElemWhat::BattlefieldInsertPos(insert_pos) => Some(insert_pos),
					_ => None,
				};

				if let (Some(selected_card_where_in_hand), Some(targeted_insert_pos)) =
					(selected_card_where_in_hand, targeted_insert_pos)
				{
					let card_that_is_played = self.hand.remove(selected_card_where_in_hand);

					match self
						.vis_elems
						.get_mut(&card_that_is_played.vis_elem_id)
						.unwrap()
						.what
					{
						VisElemWhat::Card { ref mut where_in_hand, .. } => {
							*where_in_hand = None;
						},
						_ => panic!(),
					}
					self
						.vis_elems
						.get_mut(&card_that_is_played.vis_elem_id)
						.unwrap()
						.selectable = false;

					let entity = match card_that_is_played {
						CardInHand { card: Card::Entity(entity), .. } => entity,
					};
					match targeted_insert_pos {
						BattlefieldInsertPos(BattlefieldPos::Friend(friend_i)) => {
							self.battlefield.friends.insert(
								friend_i,
								EntityOnBattlefield {
									entity,
									vis_elem_id: card_that_is_played.vis_elem_id,
								},
							);
						},
						_ => unimplemented!(),
					}
					self.update_pos_of_cards_in_hand();
					self.update_pos_of_entities_in_battlefield();
				}
			}

			self.selected_vis_elem_id = None;

			for (_id, vis_elem) in self.vis_elems.iter_mut() {
				vis_elem.targetable = false;
			}
		}
		Ok(())
	}

	fn key_down_event(
		&mut self,
		ctx: &mut Context,
		input: ggez::input::keyboard::KeyInput,
		repeated: bool,
	) -> GameResult {
		if repeated {
			return Ok(());
		}

		if let Some(keycode) = input.keycode {
			match keycode {
				KeyCode::Space => {},
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
						let dist_if_full_motion = (cur_top_left + motion).distance(dst_top_left);
						let cur_dist = cur_top_left.distance(dst_top_left);

						if dst_top_left == cur_top_left || dist_if_full_motion >= cur_dist {
							vis_elem.pos = dst_pos.clone();
							finished_animation_indices.push(animation_i);
						} else {
							vis_elem.pos.parent = None;
							vis_elem.pos.pin_point = PinPoint::TOP_LEFT;
							vis_elem.pos.in_parent_pin_point = PinPoint::TOP_LEFT;
							vis_elem.pos.coords = Vec2::from(cur_rect.point()) + motion;
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

		// Draw a line from the selected card (if any) to the cursor to make it clear that
		// we are going to do something with the selected card and whatever is going to be
		// under the cursor when we release the mouse button.
		if let (Some(selected_vis_elem_id), Some(cursor_pos)) =
			(self.selected_vis_elem_id, self.cursor_pos)
		{
			let selected_vis_elem = self.vis_elems.get(&selected_vis_elem_id).unwrap();
			let rect = self.vis_elem_actual_rect(
				selected_vis_elem.pos.clone(),
				selected_vis_elem.what.dimensions(),
			);
			let selected_vis_elem_center = rect.center();
			let line = Mesh::new_line(
				ctx,
				&[selected_vis_elem_center, cursor_pos.into()],
				12.0,
				Color::from_rgb(255, 150, 180),
			)?;
			canvas.draw(&line, Vec2::new(0.0, 0.0));
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
				.dimensions(1200.0, 800.0)
				.maximized(true),
		)
		.build()?;
	let game = Game::new(&ctx)?;
	// Lets gooooooo!! Frog Dream!!! Yaaay ^^
	ggez::event::run(ctx, event_loop, game)
}
