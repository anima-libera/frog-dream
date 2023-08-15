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
	targetable: bool,
	what: InterfaceElementWhat,
}

struct Game {
	spritesheet: Image,
	canvas_size: (f32, f32),
	battlefield: Battlefield,
	hand: Vec<Card>,
	interface_elements: Vec<InterfaceElement>,
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
			cursor_pos: None,
			hovered_card: None,
			selected_card: None,
		})
	}

	fn card_rect(&self, which_card: WhichCard) -> Rect {
		match which_card {
			WhichCard::BattlefieldFriend(i) => Rect::new(
				self.canvas_size.0 / 2.0 - 40.0 - (CardSpec::DIMS.0 + 10.0) * (i as f32 + 1.0),
				100.0,
				CardSpec::DIMS.0,
				CardSpec::DIMS.1,
			),
			WhichCard::BattlefieldFoe(i) => Rect::new(
				self.canvas_size.0 / 2.0 + 40.0 + (CardSpec::DIMS.0 + 10.0) * i as f32,
				100.0,
				CardSpec::DIMS.0,
				CardSpec::DIMS.1,
			),
			WhichCard::Hand(i) => Rect::new(
				self.canvas_size.0 / 2.0 - (CardSpec::DIMS.0 + 10.0) / 2.0 * self.hand.len() as f32
					+ (CardSpec::DIMS.0 + 10.0) * i as f32,
				500.0,
				CardSpec::DIMS.0,
				CardSpec::DIMS.1,
			),
		}
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
			let targetable = false;
			let what = InterfaceElementWhat::Creature(which_card);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected,
				targetable,
				what,
			});
		}
		for (i, _creature) in self.battlefield.foes.iter().enumerate() {
			let which_card = WhichCard::BattlefieldFoe(i);
			let rect = self.card_rect(which_card);
			let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
			let selected = self.selected_card == Some(which_card);
			let targetable = false;
			let what = InterfaceElementWhat::Creature(which_card);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected,
				targetable,
				what,
			});
		}

		for (i, _card) in self.hand.iter().enumerate() {
			let which_card = WhichCard::Hand(i);
			let rect = self.card_rect(which_card);
			let hovered = self.cursor_pos.is_some_and(|pos| rect.contains(pos));
			let selected = self.selected_card == Some(which_card);
			let targetable = false;
			let what = InterfaceElementWhat::Card(which_card);
			self.interface_elements.push(InterfaceElement {
				rect,
				hovered,
				selected,
				targetable,
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
				let targetable = true;
				let what = InterfaceElementWhat::FriendInsertionPossibility(i);
				self.interface_elements.push(InterfaceElement {
					rect,
					hovered,
					selected,
					targetable,
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
		if let event::MouseButton::Left = button {
			for interface_element in &self.interface_elements {
				if interface_element.hovered {
					if let (
						InterfaceElementWhat::FriendInsertionPossibility(dst_friend_index),
						Some(WhichCard::Hand(src_hand_index)),
					) = (&interface_element.what, self.selected_card)
					{
						let card = self.hand.remove(src_hand_index);
						self
							.battlefield
							.friends
							.insert(*dst_friend_index, Creature { card_spec: card.card_spec });
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
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

		self.draw_interface(ctx, &mut canvas)?;

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
