use std::{env, process};

use azalea::{
    entity::{PlayerAbilities, Position},
    physics::local_player::WalkDirection,
    prelude::*,
    protocol::packets::game::ServerboundPlayerAbilities,
};

const TARGET_Y: f64 = 200.0;
const ASCEND_SPEED: f64 = 0.7;
const CRUISE_SPEED: f64 = 0.7;

#[tokio::main]
async fn main() -> AppExit {
    let args = Args::parse_or_exit();
    let account = Account::offline(&args.username);

    ClientBuilder::new()
        .set_handler(handle)
        .set_state(State {
            degree: args.degree,
            dimension: args.dimension,
        })
        .start(account, args.server)
        .await
}

#[derive(Clone, Debug)]
enum Dimension {
    Overworld,
    Nether,
    End,
}

#[derive(Clone, Debug)]
struct Args {
    degree: f32,
    dimension: Dimension,
    server: String,
    username: String,
}

impl Args {
    fn parse_or_exit() -> Self {
        let mut args = env::args().skip(1);
        Self {
            degree: args.next().unwrap_or("0.0".to_owned()).parse().unwrap_or(0.0),
            dimension: args
                .next()
                .unwrap_or("overworld".to_owned())
                .parse()
                .unwrap_or_else(|message| {
                    eprintln!("{message}");
                    print_usage_and_exit();
                }),
            server: args.next().unwrap_or_else(|| "localhost".to_owned()),
            username: args.next().unwrap_or_else(|| "flying_bot".to_owned()),
        }
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: cargo run -- <degree> <dimension> [server] [username]");
    process::exit(2);
}

impl std::str::FromStr for Dimension {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "overworld" => Ok(Self::Overworld),
            "nether" => Ok(Self::Nether),
            "end" => Ok(Self::End),
            other => Err(format!("Invalid dimension '{other}'")),
        }
    }
}

#[derive(Clone, Component, Debug)]
struct State {
    degree: f32,
    dimension: Dimension,
}

impl Default for State {
    fn default() -> Self {
        Self {
            degree: 0.0,
            dimension: Dimension::Overworld,
        }
    }
}

async fn handle(bot: Client, event: Event, state: State) -> eyre::Result<()> {
    match event {
        Event::Login => {
            bot.chat("/gamemode creative");
            match state.dimension {
                Dimension::Overworld => {
                    bot.chat("/steel tp @p overworld");
                }
                Dimension::Nether => {
                    bot.chat("/steel tp @p the_nether");}
                Dimension::End => {
                    bot.chat("/steel tp @p the_end");}
            }
        }
        Event::Tick => {
            enable_flying(&bot);

            if bot.position().y < TARGET_Y {
                ascend(&bot, state.degree);
            } else {
                bot.set_jumping(false);
                cruise(&bot, state.degree);
            }
        }
        _ => {}
    }

    Ok(())
}

fn enable_flying(bot: &Client) {
    bot.query_self::<&mut PlayerAbilities, _>(|mut abilities| {
        abilities.can_fly = true;
        abilities.flying = true;
        if abilities.flying_speed == 0.0 {
            abilities.flying_speed = 0.05;
        }
    });

    bot.write_packet(ServerboundPlayerAbilities { is_flying: true });
}

fn ascend(bot: &Client, degree: f32) {
    bot.query_self::<(&mut Position, &mut azalea::entity::Physics), _>(|(mut pos, mut physics)| {
        pos.y = (pos.y + ASCEND_SPEED).min(TARGET_Y);
        physics.velocity = azalea::Vec3::ZERO;
        physics.set_on_ground(false);
        physics.set_last_on_ground(false);
        physics.no_jump_delay = 0;
    });

    bot.set_direction(degree, -90.0);
    bot.set_jumping(false);
    bot.walk(WalkDirection::None);
}

fn cruise(bot: &Client, degree: f32) {
    let yaw = f64::from(degree).to_radians();
    let dx = -yaw.sin() * CRUISE_SPEED;
    let dz = yaw.cos() * CRUISE_SPEED;

    bot.query_self::<(&mut Position, &mut azalea::entity::Physics), _>(|(mut pos, mut physics)| {
        pos.y = TARGET_Y;
        pos.x += dx;
        pos.z += dz;
        physics.velocity = azalea::Vec3::ZERO;
        physics.set_on_ground(false);
        physics.set_last_on_ground(false);
        physics.no_jump_delay = 0;
    });
    println!("{:?}", bot.position());

    bot.set_direction(degree, 0.0);
    bot.walk(WalkDirection::None);
}
