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
            direction: args.direction,
        })
        .start(account, args.server)
        .await
}

#[derive(Clone, Debug)]
struct Args {
    direction: Direction,
    server: String,
    username: String,
}

impl Args {
    fn parse_or_exit() -> Self {
        let mut args = env::args().skip(1);
        let Some(direction) = args.next() else {
            print_usage_and_exit();
        };

        let direction = direction.parse().unwrap_or_else(|message| {
            eprintln!("{message}");
            print_usage_and_exit();
        });

        Self {
            direction,
            server: args.next().unwrap_or_else(|| "localhost".to_owned()),
            username: args.next().unwrap_or_else(|| "flying_bot".to_owned()),
        }
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: cargo run -- <east|west|south|north> [server] [username]");
    process::exit(2);
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    East,
    West,
    South,
    North,
}

impl Direction {
    fn yaw(self) -> f32 {
        match self {
            Self::South => 0.0,
            Self::West => 90.0,
            Self::North => 180.0,
            Self::East => -90.0,
        }
    }

    fn delta(self) -> (f64, f64) {
        match self {
            Self::East => (CRUISE_SPEED, 0.0),
            Self::West => (-CRUISE_SPEED, 0.0),
            Self::South => (0.0, CRUISE_SPEED),
            Self::North => (0.0, -CRUISE_SPEED),
        }
    }
}

impl std::str::FromStr for Direction {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "east" => Ok(Self::East),
            "west" => Ok(Self::West),
            "south" => Ok(Self::South),
            "north" => Ok(Self::North),
            other => Err(format!("Invalid direction '{other}'")),
        }
    }
}

#[derive(Clone, Component, Debug)]
struct State {
    direction: Direction,
}

impl Default for State {
    fn default() -> Self {
        Self {
            direction: Direction::East,
        }
    }
}

async fn handle(bot: Client, event: Event, state: State) -> eyre::Result<()> {
    match event {
        Event::Login => {
            bot.chat("/gamemode creative");
        }
        Event::Tick => {
            enable_flying(&bot);

            if bot.position().y < TARGET_Y {
                ascend(&bot, state.direction);
            } else {
                bot.set_jumping(false);
                cruise(&bot, state.direction);
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

fn ascend(bot: &Client, direction: Direction) {
    bot.query_self::<(&mut Position, &mut azalea::entity::Physics), _>(|(mut pos, mut physics)| {
        pos.y = (pos.y + ASCEND_SPEED).min(TARGET_Y);
        physics.velocity = azalea::Vec3::ZERO;
        physics.set_on_ground(false);
        physics.set_last_on_ground(false);
        physics.no_jump_delay = 0;
    });

    bot.set_direction(direction.yaw(), -90.0);
    bot.set_jumping(false);
    bot.walk(WalkDirection::None);
}

fn cruise(bot: &Client, direction: Direction) {
    let (dx, dz) = direction.delta();

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

    bot.set_direction(direction.yaw(), 0.0);
    bot.walk(WalkDirection::None);
}
