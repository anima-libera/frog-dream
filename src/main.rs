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
}

impl CardSpec {
	fn draw(
		&self,
		ctx: &mut Context,
		canvas: &mut Canvas,
		spritesheet: &Image,
		dst: Vec2,
	) -> GameResult {
		let rectangle = graphics::Mesh::new_rectangle(
			ctx,
			graphics::DrawMode::stroke(3.0),
			Rect::new(dst.x, dst.y, 200.0, 250.0),
			Color::WHITE,
		)?;
		canvas.draw(&rectangle, Vec2::new(0.0, 0.0));

		let (name, sprite) = match self {
			CardSpec::Fwog => ("fwog", Rect::new(0.0, 0.2, 0.5, 0.4)),
			CardSpec::DragonFly => ("dragon fly", Rect::new(0.5, 0.12, 0.5, 0.45)),
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

struct Game {
	spritesheet: Image,
}

impl Game {
	fn new(ctx: &Context) -> GameResult<Game> {
		Ok(Game {
			spritesheet: Image::from_bytes(ctx, include_bytes!("../assets/spritesheet.png"))?,
		})
	}
}

impl event::EventHandler<ggez::GameError> for Game {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

		CardSpec::Fwog.draw(
			ctx,
			&mut canvas,
			&self.spritesheet,
			Vec2 { x: 100.0, y: 100.0 },
		)?;
		CardSpec::DragonFly.draw(
			ctx,
			&mut canvas,
			&self.spritesheet,
			Vec2 { x: 320.0, y: 100.0 },
		)?;

		canvas.finish(ctx)?;
		Ok(())
	}
}

fn main() -> GameResult {
	let cb = ggez::ContextBuilder::new("dream_frog", "Anima")
		.window_setup(WindowSetup::default().title("Dream Frog"))
		.window_mode(
			WindowMode::default()
				.resizable(true)
				.dimensions(1200.0, 800.0),
		);
	let (ctx, event_loop) = cb.build()?;
	let game = Game::new(&ctx)?;
	event::run(ctx, event_loop, game)
}
