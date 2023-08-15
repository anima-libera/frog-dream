use std::time::Duration;
use std::time::Instant;

use ggez::conf::WindowMode;
use ggez::conf::WindowSetup;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::DrawParam;
use ggez::graphics::{self, Canvas, Color, Image, Rect};
use ggez::{Context, GameResult};

enum CardSpec {
	Fwog,
	DragonFly,
	Food,
}

impl CardSpec {
	fn is_creature(&self) -> bool {
		match self {
			CardSpec::Fwog | CardSpec::DragonFly => true,
			CardSpec::Food => false,
		}
	}

	const DIMS: (f32, f32) = (200.0, 250.0);

	fn draw(
		&self,
		ctx: &mut Context,
		canvas: &mut Canvas,
		spritesheet: &Image,
		dst: Vec2,
		hovered: bool,
		selected: bool,
	) -> GameResult {
		let rectangle = graphics::Mesh::new_rectangle(
			ctx,
			graphics::DrawMode::stroke(3.0),
			Rect::new(dst.x, dst.y, CardSpec::DIMS.0, CardSpec::DIMS.1),
			if hovered { Color::YELLOW } else { Color::WHITE },
		)?;
		canvas.draw(&rectangle, Vec2::new(0.0, 0.0));
		if selected {
			let rectangle = graphics::Mesh::new_rectangle(
				ctx,
				graphics::DrawMode::stroke(9.0),
				Rect::new(dst.x, dst.y, CardSpec::DIMS.0, CardSpec::DIMS.1),
				Color::from_rgb(255, 150, 180),
			)?;
			canvas.draw(&rectangle, Vec2::new(0.0, 0.0));
		}

		let (name, sprite) = match self {
			CardSpec::Fwog => ("fwog", Rect::new(0.0, 0.2, 0.5, 0.4)),
			CardSpec::DragonFly => ("dragon fly", Rect::new(0.5, 0.12, 0.5, 0.45)),
			CardSpec::Food => ("food!", Rect::new(0.0, 0.62, 0.5, 0.38)),
		};

		canvas.draw(
			graphics::Text::new(name).set_scale(26.0),
			graphics::DrawParam::from(Vec2::new(dst.x + 10.0, dst.y + 10.0)).color(Color::WHITE),
		);
		canvas.draw(
			spritesheet,
			DrawParam::default()
				.dest(Vec2::new(dst.x + 10.0, dst.y + 30.0 + 10.0))
				.scale(Vec2::new(0.1, 0.1))
				.src(sprite),
		);

		Ok(())
	}
}

struct Creature {
	card_spec: CardSpec,
}

struct Battlefield {
	friends: Vec<Creature>,
	foes: Vec<Creature>,
}

struct Card {
	card_spec: CardSpec,
}

enum InterfaceElementWhat {
	// TODO: Use something else than `WhichCard` in these!
	// TODO: Remove `WhichCard` from the code, it will cause confusion.
	Card(WhichCard),
	Creature(WhichCard),
	FriendInsertionPossibility(usize),
}

struct InterfaceElement {
	rect: Rect,
	hovered: bool,
	selected: bool,
	_targetable: bool,
	what: InterfaceElementWhat,
}

struct TimeProgression {
	start: Instant,
	duration: Duration,
}

impl TimeProgression {
	fn with_duration(duration: Duration) -> TimeProgression {
		TimeProgression { start: Instant::now(), duration }
	}

	fn progression(&self) -> f32 {
		self.start.elapsed().as_secs_f32() / self.duration.as_secs_f32()
	}
}

enum AnimationWhat {
	PlacingCreatureFromHand {
		src_hand_index: usize,
		dst_friend_index: usize,
		card: Card,
		src_point: Vec2,
		dst_point: Vec2,
	},
}

struct Animation {
	tp: TimeProgression,
	what: AnimationWhat,
}

struct Game {
	spritesheet: Image,
	canvas_size: (f32, f32),
	battlefield: Battlefield,
	hand: Vec<Card>,
	interface_elements: Vec<InterfaceElement>,
	animations: Vec<Animation>,
	cursor_pos: Option<Vec2>,
	hovered_card: Option<WhichCard>,
	selected_card: Option<WhichCard>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WhichCard {
	BattlefieldFriend(usize),
	BattlefieldFoe(usize),
	Hand(usize),
}

impl Game {
	fn new(ctx: &Context) -> GameResult<Game> {
		let friends = vec![Creature { card_spec: CardSpec::Fwog }];
		let foes = vec![Creature { card_spec: CardSpec::DragonFly }];
		let hand = vec![
			Card { card_spec: CardSpec::Fwog },
			Card { card_spec: CardSpec::Fwog },
			Card { card_spec: CardSpec::DragonFly },
			Card { card_spec: CardSpec::Food },
		];
		Ok(Game {
			spritesheet: Image::from_bytes(ctx, include_bytes!("../assets/spritesheet.png"))?,
			canvas_size: ctx.gfx.size(),
			battlefield: Battlefield { friends, foes },
			hand,
			interface_elements: vec![],
			animations: vec![],
			cursor_pos: None,
			hovered_card: None,
			selected_card: None,
		})
	}

	fn card_rect(&self, which_card: WhichCard) -> Rect {
		let x = match which_card {
			WhichCard::BattlefieldFriend(i) => {
				let mut animations_offset = 0.0;
				for animation in self.animations.iter() {
					match animation.what {
						AnimationWhat::PlacingCreatureFromHand {
							src_hand_index,
							dst_friend_index,
							..
						} => {
							if dst_friend_index <= i {
								animations_offset += (CardSpec::DIMS.0 + 10.0) * animation.tp.progression();
							}
						},
					}
				}
				self.canvas_size.0 / 2.0
					- 40.0 - (CardSpec::DIMS.0 + 10.0) * (i as f32 + 1.0)
					- animations_offset
			},
			WhichCard::BattlefieldFoe(i) => {
				self.canvas_size.0 / 2.0 + 40.0 + (CardSpec::DIMS.0 + 10.0) * i as f32
			},
			WhichCard::Hand(i) => {
				let mut animations_offset_len = 0.0;
				let mut animations_offset = 0.0;
				for animation in self.animations.iter() {
					match animation.what {
						AnimationWhat::PlacingCreatureFromHand {
							src_hand_index,
							dst_friend_index,
							..
						} => {
							animations_offset_len += 1.0 - animation.tp.progression();
							if src_hand_index <= i {
								animations_offset +=
									(CardSpec::DIMS.0 + 10.0) * (1.0 - animation.tp.progression());
							}
						},
					}
				}
				self.canvas_size.0 / 2.0
					- (CardSpec::DIMS.0 + 10.0) / 2.0 * (self.hand.len() as f32 + animations_offset_len)
					+ (CardSpec::DIMS.0 + 10.0) * i as f32
					+ animations_offset
			},
		};
		let y = match which_card {
			WhichCard::BattlefieldFriend(_i) => 100.0,
			WhichCard::BattlefieldFoe(_i) => 100.0,
			WhichCard::Hand(_i) => 500.0,
		};

		Rect::new(x, y, CardSpec::DIMS.0, CardSpec::DIMS.1)
	}

	fn all_cards(&self) -> Vec<WhichCard> {
		let mut cards = vec![];
		for i in 0..self.battlefield.friends.len() {
			cards.push(WhichCard::BattlefieldFriend(i));
		}
		for i in 0..self.battlefield.foes.len() {
			cards.push(WhichCard::BattlefieldFoe(i));
		}
		for i in 0..self.hand.len() {
			cards.push(WhichCard::Hand(i));
		}
		cards
	}

	fn refresh_interface(&mut self) {
		self.interface_elements.clear();

		for (i, _creature) in self.battlefield.friends.iter().enumerate() {
			let which_card = WhichCard::BattlefieldFriend(i);
			let rect = self.card_rect(which_card);
			let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
			let selected = self.selected_card == Some(which_card);
			let what = InterfaceElementWhat::Creature(which_card);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected,
				_targetable: false,
				what,
			});
		}
		for (i, _creature) in self.battlefield.foes.iter().enumerate() {
			let which_card = WhichCard::BattlefieldFoe(i);
			let rect = self.card_rect(which_card);
			let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
			let selected = self.selected_card == Some(which_card);
			let what = InterfaceElementWhat::Creature(which_card);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected,
				_targetable: false,
				what,
			});
		}

		for (i, _card) in self.hand.iter().enumerate() {
			let which_card = WhichCard::Hand(i);
			let rect = self.card_rect(which_card);
			let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
			let selected = self.selected_card == Some(which_card);
			let what = InterfaceElementWhat::Card(which_card);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected,
				_targetable: false,
				what,
			});
		}

		let display_insert_possibilities = if let Some(WhichCard::Hand(i)) = self.selected_card {
			let selected_card = &self.hand[i];
			selected_card.card_spec.is_creature()
		} else {
			false
		};
		if display_insert_possibilities {
			for i in 0..(self.battlefield.friends.len() + 1) {
				let x = self.card_rect(WhichCard::BattlefieldFriend(i)).right() + 5.0;
				let w = 50.0;
				let rect = Rect::new(x - w / 2.0, 100.0 + CardSpec::DIMS.1 + 10.0, w, 50.0);
				let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
				let selected = false;
				let what = InterfaceElementWhat::FriendInsertionPossibility(i);
				self.interface_elements.push(InterfaceElement {
					rect,
					hovered,
					selected,
					_targetable: true,
					what,
				});
			}
		}
	}

	fn draw_interface(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
		for elem in self.interface_elements.iter() {
			match elem.what {
				InterfaceElementWhat::Card(which_card) => {
					let card = match which_card {
						WhichCard::Hand(i) => &self.hand[i],
						_ => panic!("bug: cards are supposed to be in the hand"),
					};
					card.card_spec.draw(
						ctx,
						canvas,
						&self.spritesheet,
						elem.rect.point().into(),
						elem.hovered,
						elem.selected,
					)?;
				},
				InterfaceElementWhat::Creature(which_card) => {
					let card = match which_card {
						WhichCard::BattlefieldFriend(i) => &self.battlefield.friends[i],
						WhichCard::BattlefieldFoe(i) => &self.battlefield.foes[i],
						_ => panic!("bug: creatures are supposed to be on the battle field"),
					};
					card.card_spec.draw(
						ctx,
						canvas,
						&self.spritesheet,
						elem.rect.point().into(),
						elem.hovered,
						elem.selected,
					)?;
				},
				InterfaceElementWhat::FriendInsertionPossibility(_index) => {
					let rectangle = graphics::Mesh::new_polyline(
						ctx,
						graphics::DrawMode::stroke(3.0),
						&[
							Vec2::new(elem.rect.center().x, elem.rect.top()),
							Vec2::new(elem.rect.left(), elem.rect.bottom()),
							Vec2::new(elem.rect.right(), elem.rect.bottom()),
							Vec2::new(elem.rect.center().x, elem.rect.top()),
						],
						if elem.hovered {
							Color::YELLOW
						} else {
							Color::WHITE
						},
					)?;
					canvas.draw(&rectangle, Vec2::new(0.0, 0.0));
				},
			}
		}
		Ok(())
	}
}

fn lerp(progression: f32, start_value: f32, end_value: f32) -> f32 {
	start_value + progression * (end_value - start_value)
}

impl event::EventHandler<ggez::GameError> for Game {
	fn mouse_motion_event(
		&mut self,
		_ctx: &mut Context,
		x: f32,
		y: f32,
		_dx: f32,
		_dy: f32,
	) -> GameResult {
		self.cursor_pos = Some(Vec2::new(x, y));
		self.hovered_card = None;
		for card in self.all_cards() {
			if self.card_rect(card).contains(Vec2::new(x, y)) {
				self.hovered_card = Some(card);
				break;
			}
		}
		self.refresh_interface();
		Ok(())
	}

	fn mouse_button_down_event(
		&mut self,
		_ctx: &mut Context,
		button: event::MouseButton,
		_x: f32,
		_y: f32,
	) -> GameResult {
		if !self.animations.is_empty() {
			return Ok(());
		}
		if let event::MouseButton::Left = button {
			self.selected_card = self.hovered_card;
		}
		self.refresh_interface();
		Ok(())
	}

	fn mouse_button_up_event(
		&mut self,
		_ctx: &mut Context,
		button: event::MouseButton,
		_x: f32,
		_y: f32,
	) -> GameResult {
		if !self.animations.is_empty() {
			return Ok(());
		}
		if let event::MouseButton::Left = button {
			for interface_element in &self.interface_elements {
				if interface_element.hovered {
					if let (
						InterfaceElementWhat::FriendInsertionPossibility(dst_friend_index),
						Some(WhichCard::Hand(src_hand_index)),
					) = (&interface_element.what, self.selected_card)
					{
						let dst_friend_index = *dst_friend_index;
						let src_point = self
							.card_rect(WhichCard::Hand(src_hand_index))
							.point()
							.into();
						let dst_point = self
							.card_rect(WhichCard::BattlefieldFriend(dst_friend_index))
							.point()
							.into();
						let card = self.hand.remove(src_hand_index);
						let duration = Duration::from_secs_f32(0.2);
						self.animations.push(Animation {
							tp: TimeProgression::with_duration(duration),
							what: AnimationWhat::PlacingCreatureFromHand {
								src_hand_index,
								dst_friend_index,
								card,
								src_point,
								dst_point,
							},
						});
					}
				}
			}
			self.selected_card = None;
		}
		self.refresh_interface();
		Ok(())
	}

	fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
		self.canvas_size = (width, height);
		self.refresh_interface();
		Ok(())
	}

	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		let animations_were_going_on = !self.animations.is_empty();

		// Handle the end of animations.
		let mut ending_animation_indices = vec![];
		for (i, animation) in self.animations.iter().enumerate() {
			if animation.tp.progression() >= 1.0 {
				// Animation is finished, we have to apply its final effects and remove it.
				ending_animation_indices.push(i);
			}
		}
		for i in ending_animation_indices.into_iter().rev() {
			let animation = self.animations.remove(i);
			match animation.what {
				AnimationWhat::PlacingCreatureFromHand { dst_friend_index, card, .. } => {
					self
						.battlefield
						.friends
						.insert(dst_friend_index, Creature { card_spec: card.card_spec });
				},
			}
		}

		if animations_were_going_on {
			self.refresh_interface();
		}

		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

		self.draw_interface(ctx, &mut canvas)?;

		// TODO: Somehow move this into `Game::refresh_interface` and produce an `InterfaceElement`
		// instead of directly drawing a card.
		for animation in self.animations.iter() {
			let progression = animation.tp.progression();
			match &animation.what {
				AnimationWhat::PlacingCreatureFromHand { card, src_point, dst_point, .. } => {
					let pos = Vec2::new(
						lerp(progression, src_point.x, dst_point.x),
						lerp(progression, src_point.y, dst_point.y),
					);
					card
						.card_spec
						.draw(ctx, &mut canvas, &self.spritesheet, pos, false, false)?;
				},
			}
		}

		if let Some(selected_card) = self.selected_card {
			let selected_card_pos = self.card_rect(selected_card).center();
			let cursor_pos = ctx.mouse.position();
			let line = graphics::Mesh::new_line(
				ctx,
				&[selected_card_pos, cursor_pos],
				12.0,
				Color::from_rgb(255, 150, 180),
			)?;
			canvas.draw(&line, Vec2::new(0.0, 0.0));
		}

		canvas.finish(ctx)?;
		Ok(())
	}
}

fn main() -> GameResult {
	let cb = ggez::ContextBuilder::new("frog_dream", "Anima")
		.window_setup(WindowSetup::default().title("Frog Dream"))
		.window_mode(
			WindowMode::default()
				.resizable(true)
				.dimensions(1200.0, 900.0),
		);
	let (ctx, event_loop) = cb.build()?;
	let game = Game::new(&ctx)?;
	event::run(ctx, event_loop, game)
}
