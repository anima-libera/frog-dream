use ggez::conf::WindowSetup;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::DrawParam;
use ggez::graphics::{self, Color, Image};
use ggez::{Context, GameResult};

struct Game {
	frog_image: Image,
}

impl Game {
	fn new(ctx: &Context) -> GameResult<Game> {
		Ok(Game {
			frog_image: Image::from_bytes(ctx, include_bytes!("../assets/spritesheet.png"))?,
		})
	}
}

impl event::EventHandler<ggez::GameError> for Game {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas =
			graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

		let rectangle = graphics::Mesh::new_rectangle(
			ctx,
			graphics::DrawMode::stroke(3.0),
			graphics::Rect::new(100.0, 100.0, 200.0, 250.0),
			Color::WHITE,
		)?;
		canvas.draw(&rectangle, Vec2::new(0.0, 0.0));

		canvas.draw(
			graphics::Text::new("fwog").set_scale(26.0),
			graphics::DrawParam::from(Vec2::new(110.0, 110.0)).color(Color::WHITE),
		);

		canvas.draw(
			&self.frog_image,
			DrawParam::default()
				.dest(Vec2::new(110.0, 140.0))
				.scale(Vec2::new(0.1, 0.1))
				.src(graphics::Rect::new(0.0, 0.2, 0.5, 0.4)),
		);

		canvas.finish(ctx)?;
		Ok(())
	}
}

fn main() -> GameResult {
	let cb = ggez::ContextBuilder::new("dream_frog", "Anima")
		.window_setup(WindowSetup::default().title("Dream Frog"));
	let (ctx, event_loop) = cb.build()?;
	let game = Game::new(&ctx)?;
	event::run(ctx, event_loop, game)
}
