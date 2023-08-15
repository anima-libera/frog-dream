use std::time::Duration;
use std::time::Instant;

use ggez::glam::*;
use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, Image, Mesh, Rect, Text};
use ggez::{Context, GameResult};

#[derive(Clone)]
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
		let rectangle = Mesh::new_rectangle(
			ctx,
			DrawMode::stroke(3.0),
			Rect::new(dst.x, dst.y, CardSpec::DIMS.0, CardSpec::DIMS.1),
			if hovered { Color::YELLOW } else { Color::WHITE },
		)?;
		canvas.draw(&rectangle, Vec2::new(0.0, 0.0));
		if selected {
			let rectangle = Mesh::new_rectangle(
				ctx,
				DrawMode::stroke(9.0),
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
			Text::new(name).set_scale(26.0),
			DrawParam::from(Vec2::new(dst.x + 10.0, dst.y + 10.0)).color(Color::WHITE),
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

#[derive(Clone)]
struct Card {
	card_spec: CardSpec,
}

enum InterfaceElementWhat {
	HandCard(WhichHandCard),
	Creature(WhichBattlefieldCreature),
	/// A targetable arrow that point to a space on the battlefield where a friend creature
	/// could be placed. It contains the index that the creature will have in the friend vec
	/// if placed here.
	FriendInsertionSlot(usize),
	/// A card floating around (for example when moving during an animation).
	Card(Card),
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

/// The game! Its here ^^
struct Game {
	spritesheet: Image,
	canvas_size: (f32, f32),
	battlefield: Battlefield,
	hand: Vec<Card>,
	selected_hand_card: Option<WhichHandCard>,
	interface_elements: Vec<InterfaceElement>,
	animation: Option<Animation>,
	cursor_pos: Option<Vec2>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct WhichBattlefieldFriend(usize);
#[derive(Clone, Copy, PartialEq, Eq)]
struct WhichBattlefieldFoe(usize);
#[derive(Clone, Copy, PartialEq, Eq)]
enum WhichBattlefieldCreature {
	Friend(WhichBattlefieldFriend),
	Foe(WhichBattlefieldFoe),
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct WhichHandCard(usize);

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
			selected_hand_card: None,
			interface_elements: vec![],
			animation: None,
			cursor_pos: None,
		})
	}

	fn creature_rect(&self, which_creature: WhichBattlefieldCreature) -> Rect {
		let x = match which_creature {
			WhichBattlefieldCreature::Friend(WhichBattlefieldFriend(i)) => {
				// If a creature is being placed, other creatures might have to make space, smoothly.
				let animation_offset = if let Some(Animation {
					tp,
					what: AnimationWhat::PlacingCreatureFromHand { dst_friend_index, .. },
				}) = &self.animation
				{
					// A creature is being placed...
					if *dst_friend_index <= i {
						// ... and the one we are intrested in here has to make space.
						(CardSpec::DIMS.0 + 10.0) * tp.progression()
					} else {
						// ... and the one we are intrested in here happens to not have to move.
						0.0
					}
				} else {
					0.0
				};

				self.canvas_size.0 / 2.0
					- 40.0 - (CardSpec::DIMS.0 + 10.0) * (i as f32 + 1.0)
					- animation_offset
			},

			WhichBattlefieldCreature::Foe(WhichBattlefieldFoe(i)) => {
				self.canvas_size.0 / 2.0 + 40.0 + (CardSpec::DIMS.0 + 10.0) * i as f32
			},
		};

		Rect::new(x, 100.0, CardSpec::DIMS.0, CardSpec::DIMS.1)
	}

	fn hand_card_rect(&self, which_hand_card: WhichHandCard) -> Rect {
		let x = match which_hand_card {
			WhichHandCard(i) => {
				// If a creature is being placed, other hand cards might have fill the gap, smoothly.
				// `animation_len_offset` is an offset to the length (in number of cards) of the hand,
				// so that it can behaves like the number of cards is decremented but smoothly.
				let (animation_offset, animation_len_offset) = if let Some(Animation {
					tp,
					what: AnimationWhat::PlacingCreatureFromHand { src_hand_index, .. },
				}) = &self.animation
				{
					// A creature is being placed...
					if *src_hand_index <= i {
						// ... and the one we are intrested in here has to move.
						(
							(CardSpec::DIMS.0 + 10.0) * (1.0 - tp.progression()),
							1.0 - tp.progression(),
						)
					} else {
						// ... and the one we are intrested in here doesn't have to do much.
						(0.0, 1.0 - tp.progression())
					}
				} else {
					(0.0, 0.0)
				};

				self.canvas_size.0 / 2.0
					- (CardSpec::DIMS.0 + 10.0) / 2.0 * (self.hand.len() as f32 + animation_len_offset)
					+ (CardSpec::DIMS.0 + 10.0) * i as f32
					+ animation_offset
			},
		};

		Rect::new(x, 500.0, CardSpec::DIMS.0, CardSpec::DIMS.1)
	}

	fn refresh_interface(&mut self) {
		self.interface_elements.clear();

		for (i, _creature) in self.battlefield.friends.iter().enumerate() {
			let which_creature = WhichBattlefieldCreature::Friend(WhichBattlefieldFriend(i));
			let rect = self.creature_rect(which_creature);
			let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
			let what = InterfaceElementWhat::Creature(which_creature);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected: false,
				_targetable: false,
				what,
			});
		}
		for (i, _creature) in self.battlefield.foes.iter().enumerate() {
			let which_creature = WhichBattlefieldCreature::Foe(WhichBattlefieldFoe(i));
			let rect = self.creature_rect(which_creature);
			let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
			let what = InterfaceElementWhat::Creature(which_creature);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected: false,
				_targetable: false,
				what,
			});
		}

		for (i, _card) in self.hand.iter().enumerate() {
			let which_hand_card = WhichHandCard(i);
			let rect = self.hand_card_rect(which_hand_card);
			let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
			let selected = self.selected_hand_card == Some(which_hand_card);
			let what = InterfaceElementWhat::HandCard(which_hand_card);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected,
				_targetable: false,
				what,
			});
		}

		let display_insert_possibilities = if let Some(WhichHandCard(i)) = self.selected_hand_card {
			let selected_card = &self.hand[i];
			selected_card.card_spec.is_creature()
		} else {
			false
		};
		if display_insert_possibilities {
			for i in 0..(self.battlefield.friends.len() + 1) {
				let x = self
					.creature_rect(WhichBattlefieldCreature::Friend(WhichBattlefieldFriend(i)))
					.right() + 5.0;
				let w = 50.0;
				let rect = Rect::new(x - w / 2.0, 100.0 + CardSpec::DIMS.1 + 10.0, w, 50.0);
				let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
				let selected = false;
				let what = InterfaceElementWhat::FriendInsertionSlot(i);
				self.interface_elements.push(InterfaceElement {
					rect,
					hovered,
					selected,
					_targetable: true,
					what,
				});
			}
		}

		if let Some(Animation { tp, what }) = &self.animation {
			let progression = tp.progression();
			match what {
				AnimationWhat::PlacingCreatureFromHand { card, src_point, dst_point, .. } => {
					let pos = Vec2::new(
						lerp(progression, src_point.x, dst_point.x),
						lerp(progression, src_point.y, dst_point.y),
					);
					self.interface_elements.push(InterfaceElement {
						rect: Rect::new(pos.x, pos.y, CardSpec::DIMS.0, CardSpec::DIMS.1),
						hovered: false,
						selected: false,
						_targetable: false,
						what: InterfaceElementWhat::Card(card.clone()),
					});
				},
			}
		}
	}

	fn draw_interface(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
		for elem in self.interface_elements.iter() {
			match &elem.what {
				InterfaceElementWhat::HandCard(WhichHandCard(i)) => {
					let card = &self.hand[*i];
					card.card_spec.draw(
						ctx,
						canvas,
						&self.spritesheet,
						elem.rect.point().into(),
						elem.hovered,
						elem.selected,
					)?;
				},
				InterfaceElementWhat::Card(card) => {
					card.card_spec.draw(
						ctx,
						canvas,
						&self.spritesheet,
						elem.rect.point().into(),
						elem.hovered,
						elem.selected,
					)?;
				},
				InterfaceElementWhat::Creature(which_creature) => {
					let creature = match which_creature {
						WhichBattlefieldCreature::Friend(WhichBattlefieldFriend(i)) => {
							&self.battlefield.friends[*i]
						},
						WhichBattlefieldCreature::Foe(WhichBattlefieldFoe(i)) => {
							&self.battlefield.foes[*i]
						},
					};
					creature.card_spec.draw(
						ctx,
						canvas,
						&self.spritesheet,
						elem.rect.point().into(),
						elem.hovered,
						elem.selected,
					)?;
				},
				InterfaceElementWhat::FriendInsertionSlot(_index) => {
					let rectangle = Mesh::new_polyline(
						ctx,
						DrawMode::stroke(3.0),
						&[
							Vec2::new(elem.rect.center().x, elem.rect.top()),
							Vec2::new(elem.rect.left(), elem.rect.bottom()),
							Vec2::new(elem.rect.right(), elem.rect.bottom()),
							Vec2::new(elem.rect.center().x, elem.rect.top()),
						],
						if elem.hovered {
							Color::YELLOW
						} else {
							Color::CYAN
						},
					)?;
					canvas.draw(&rectangle, Vec2::new(0.0, 0.0));
				},
			}
		}
		Ok(())
	}

	fn place_creature_from_hand(&mut self, src_hand_index: usize, dst_friend_index: usize) {
		let src_point = self
			.hand_card_rect(WhichHandCard(src_hand_index))
			.point()
			.into();
		let dst_point = self
			.creature_rect(WhichBattlefieldCreature::Friend(WhichBattlefieldFriend(
				dst_friend_index,
			)))
			.point()
			.into();
		let card = self.hand.remove(src_hand_index);
		let duration = Duration::from_secs_f32(0.2);
		self.animation = Some(Animation {
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

fn lerp(progression: f32, start_value: f32, end_value: f32) -> f32 {
	start_value + progression * (end_value - start_value)
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
		self.refresh_interface();
		Ok(())
	}

	fn mouse_button_down_event(
		&mut self,
		_ctx: &mut Context,
		button: ggez::event::MouseButton,
		x: f32,
		y: f32,
	) -> GameResult {
		if self.animation.is_some() {
			return Ok(());
		}
		if let ggez::event::MouseButton::Left = button {
			for interface_element in self.interface_elements.iter() {
				if interface_element.rect.contains(Vec2::new(x, y)) {
					if let InterfaceElementWhat::HandCard(which_hand_card) = interface_element.what {
						self.selected_hand_card = Some(which_hand_card);
						break;
					}
				}
			}
		}
		self.refresh_interface();
		Ok(())
	}

	fn mouse_button_up_event(
		&mut self,
		_ctx: &mut Context,
		button: ggez::event::MouseButton,
		_x: f32,
		_y: f32,
	) -> GameResult {
		if self.animation.is_some() {
			return Ok(());
		}
		if let ggez::event::MouseButton::Left = button {
			for interface_element in &self.interface_elements {
				if interface_element.hovered {
					if let (
						InterfaceElementWhat::FriendInsertionSlot(dst_friend_index),
						Some(WhichHandCard(src_hand_index)),
					) = (&interface_element.what, self.selected_hand_card)
					{
						// If we get here, it means that we previously selected a creature card in hand
						// and dragged it over an insertion slot and released it.
						// It shall translate in this card's creature being placed on the battlefield
						// on the chosen spot.
						self.place_creature_from_hand(src_hand_index, *dst_friend_index);
					}
					break;
				}
			}
			self.selected_hand_card = None;
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
		let an_animation_was_going_on = self.animation.is_some();

		// When the animation is over, we remove it and apply its effects (if any) on the board.
		if let Some(Animation { tp, .. }) = &self.animation {
			if tp.progression() >= 1.0 {
				match self.animation.take().unwrap().what {
					AnimationWhat::PlacingCreatureFromHand { dst_friend_index, card, .. } => {
						self
							.battlefield
							.friends
							.insert(dst_friend_index, Creature { card_spec: card.card_spec });
					},
				}
			}
		}

		if an_animation_was_going_on {
			self.refresh_interface();
		}

		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

		self.draw_interface(ctx, &mut canvas)?;

		// Draw a line from the selected card (if any) to the cursor to make it clear that
		// we are going to do something with the selected card and whatever is going to be
		// under the cursor when we release the mouse button.
		if let Some(which_hand_card) = self.selected_hand_card {
			let card_center = self.hand_card_rect(which_hand_card).center();
			let cursor_pos = ctx.mouse.position();
			let line = Mesh::new_line(
				ctx,
				&[card_center, cursor_pos],
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
	let (ctx, event_loop) = ggez::ContextBuilder::new("frog_dream", "Anima")
		.window_setup(ggez::conf::WindowSetup::default().title("Frog Dream"))
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
