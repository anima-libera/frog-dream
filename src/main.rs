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
	const DIMS: (f32, f32) = (200.0, 250.0);

	fn draw(
		&self,
		ctx: &mut Context,
		canvas: &mut Canvas,
		spritesheet: &Image,
		dst: Vec2,
		hovered: bool,
	) -> GameResult {
		let rectangle = graphics::Mesh::new_rectangle(
			ctx,
			graphics::DrawMode::stroke(3.0),
			Rect::new(dst.x, dst.y, CardSpec::DIMS.0, CardSpec::DIMS.1),
			if hovered { Color::YELLOW } else { Color::WHITE },
		)?;
		canvas.draw(&rectangle, Vec2::new(0.0, 0.0));

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

struct Game {
	spritesheet: Image,
	battlefield: Battlefield,
	hand: Vec<Card>,
	hovered_card: Option<WhichCard>,
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
			battlefield: Battlefield { friends, foes },
			hand,
			hovered_card: None,
		})
	}

	fn card_rect(&self, ctx_size: (f32, f32), which_card: WhichCard) -> Rect {
		match which_card {
			WhichCard::BattlefieldFriend(i) => Rect::new(
				ctx_size.0 / 2.0 - 40.0 - CardSpec::DIMS.0 * (i as f32 + 1.0),
				100.0,
				CardSpec::DIMS.0,
				CardSpec::DIMS.1,
			),
			WhichCard::BattlefieldFoe(i) => Rect::new(
				ctx_size.0 / 2.0 + 40.0 + CardSpec::DIMS.0 * i as f32,
				100.0,
				CardSpec::DIMS.0,
				CardSpec::DIMS.1,
			),
			WhichCard::Hand(i) => Rect::new(
				ctx_size.0 / 2.0 - (CardSpec::DIMS.0 + 10.0) / 2.0 * self.hand.len() as f32
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
}

impl event::EventHandler<ggez::GameError> for Game {
	fn mouse_motion_event(
		&mut self,
		ctx: &mut Context,
		x: f32,
		y: f32,
		_dx: f32,
		_dy: f32,
	) -> GameResult {
		for card in self.all_cards() {
			if self
				.card_rect(ctx.gfx.size(), card)
				.contains(Vec2::new(x, y))
			{
				self.hovered_card = Some(card);
				return Ok(());
			}
		}
		self.hovered_card = None;
		Ok(())
	}

	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

		for (i, creature) in self.battlefield.friends.iter().enumerate() {
			let which_card = WhichCard::BattlefieldFriend(i);
			creature.card_spec.draw(
				ctx,
				&mut canvas,
				&self.spritesheet,
				self.card_rect(ctx.gfx.size(), which_card).point().into(),
				self
					.hovered_card
					.is_some_and(|hovered| hovered == which_card),
			)?;
		}
		for (i, creature) in self.battlefield.foes.iter().enumerate() {
			let which_card = WhichCard::BattlefieldFoe(i);
			creature.card_spec.draw(
				ctx,
				&mut canvas,
				&self.spritesheet,
				self.card_rect(ctx.gfx.size(), which_card).point().into(),
				self
					.hovered_card
					.is_some_and(|hovered| hovered == which_card),
			)?;
		}

		for (i, card) in self.hand.iter().enumerate() {
			let which_card = WhichCard::Hand(i);
			card.card_spec.draw(
				ctx,
				&mut canvas,
				&self.spritesheet,
				self.card_rect(ctx.gfx.size(), which_card).point().into(),
				self
					.hovered_card
					.is_some_and(|hovered| hovered == which_card),
			)?;
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
