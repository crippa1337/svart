use std::{
    error::Error,
    fs::File,
    io::{stdin, stdout, BufWriter, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

use cozy_chess::{Board, Color, GameStatus, Piece};

use crate::{
    definitions,
    engine::{movegen, position::is_quiet, search::Search, tt::TT},
    uci::handler::SearchType,
};

const DEFAULT: &str = "\x1b[0m";
const WHITE: &str = "\x1b[38;5;15m";
const ORANGE: &str = "\x1b[38;5;208m";
const GREEN: &str = "\x1b[38;5;40m";
const RED: &str = "\x1b[38;5;196m";

static STOP_FLAG: AtomicBool = AtomicBool::new(false);
static FENS: AtomicU64 = AtomicU64::new(0);
static WHITE_WINS: AtomicU64 = AtomicU64::new(0);
static BLACK_WINS: AtomicU64 = AtomicU64::new(0);
static DRAWS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct Parameters {
    // The number of games to generate
    games: usize,
    // The number of threads to use
    threads: usize,
    // Search type for the evaluations
    st: SearchType,
}

impl Parameters {
    fn new(games: usize, threads: usize, st: SearchType) -> Self {
        Self { games, threads, st }
    }
}

pub fn root() -> Result<(), Box<dyn Error>> {
    // Get the number of games to generate
    println!("How many games would you like to gen? [1, 100M]");
    let mut inp_games = String::new();
    stdin().read_line(&mut inp_games).unwrap();

    let games = inp_games.trim().parse::<usize>()?;
    if !(1..=100_000_000).contains(&games) {
        panic!("Invalid game range! {games}, needs to be between [1, 100M]")
    }

    // Get the number of threads to use
    println!("How many threads would you like to use? [1, 64]");
    let mut inp_threads = String::new();
    stdin().read_line(&mut inp_threads).unwrap();

    let threads = inp_threads.trim().parse::<usize>()?;
    if !(1..=64).contains(&threads) {
        panic!("Invalid thread range! {threads}, needs to be between [1, 64]")
    }

    // Get the search type for the search evaluations
    println!("What search type would you like to use? [depth, nodes]");
    let mut inp_st = String::new();
    stdin().read_line(&mut inp_st).unwrap();

    let st = match inp_st.trim().to_lowercase().as_str() {
        "depth" => {
            println!("What depth would you like to use? [1, 100]");
            let mut inp_depth = String::new();
            stdin().read_line(&mut inp_depth).unwrap();

            let depth = inp_depth.trim().parse::<i32>()?;
            if !(1..=100).contains(&depth) {
                panic!("Invalid depth range! {depth}, needs to be between [1, 100]")
            }

            SearchType::Depth(depth)
        }
        "nodes" => {
            println!("What nodes would you like to use? [1, 100M]");
            let mut inp_nodes = String::new();
            stdin().read_line(&mut inp_nodes).unwrap();

            let nodes = inp_nodes.trim().parse::<u64>()?;
            if !(1..=100_000_000).contains(&nodes) {
                panic!("Invalid nodes range! {nodes}, needs to be between [1, 100M]")
            }

            SearchType::Nodes(nodes)
        }
        _ => {
            panic!("Invalid search type! {inp_st}")
        }
    };

    // Let the user confirm the parameters
    let params = Parameters::new(games, threads, st);
    println!(
        "\n{GREEN}Confirmed parameters: {DEFAULT}[games: {WHITE}{}{DEFAULT}, threads: {WHITE}{}{DEFAULT}, search type: {WHITE}{:?}{DEFAULT}]",
        params.games, params.threads, params.st
    );
    if params.games % params.threads != 0 {
        println!("{ORANGE}WARNING: {DEFAULT}The number of games is not divisible by the number of threads!");
    }
    println!("Press enter to continue...");
    stdin().read_line(&mut String::new()).unwrap();

    generate_main(params);

    println!("We're done B)");
    Ok(())
}

fn generate_main(params: Parameters) {
    ctrlc::set_handler(move || {
        STOP_FLAG.store(true, Ordering::SeqCst);
        println!("Stopping generation...");
    })
    .expect("Failed to set CTRL+C handler.");

    let run_id = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    println!("{RED}ATTENTION: {DEFAULT}This run will be saved to data-{run_id}");
    println!(
        "{GREEN}Generating {DEFAULT}{} games with {} threads on {:?}...",
        params.games, params.threads, params.st
    );

    let data_dir = PathBuf::from("data").join(run_id);
    std::fs::create_dir_all(&data_dir).unwrap();

    std::thread::scope(|s| {
        for i in 0..params.threads {
            let path = &data_dir;
            let tprms = &params;
            s.spawn(move || {
                generate_thread(i, path, tprms);
            });
        }
    })
}

fn generate_thread(id: usize, data_dir: &Path, options: &Parameters) {
    let tt = TT::new(16);
    let mut search = Search::new(tt);
    let rng = fastrand::Rng::new();

    let mut board;
    let mut game_buffer: Vec<(i32, String)> = vec![];
    let mut hashes: Vec<u64>;

    let mut output_file = File::create(data_dir.join(format!("thread_{id}.txt"))).unwrap();
    let mut output_buffer = BufWriter::new(&mut output_file);

    let games_per_thread = (options.games / options.threads).max(1); // at least one game
    let timer = Instant::now();

    'main: for games_played in 0..games_per_thread {
        // Information print from main thread
        if id == 0 && games_played != 0 && games_played % 64 == 0 {
            let fens = FENS.load(Ordering::Relaxed);
            let elapsed = timer.elapsed().as_secs_f64();
            let fens_per_sec = fens as f64 / elapsed;

            let ww = WHITE_WINS.load(Ordering::Relaxed);
            let bw = BLACK_WINS.load(Ordering::Relaxed);
            let dr = DRAWS.load(Ordering::Relaxed);
            let tot_games = ww + bw + dr;
            let percentage = (games_played as f64 / games_per_thread as f64) * 100.0;

            let time_per_game = elapsed / games_played as f64;
            let etr = (games_per_thread - games_played) as f64 * time_per_game;

            println!("\n{GREEN}Generated {DEFAULT}{fens} FENs [{fens_per_sec:.2} FEN/s]");
            println!("{GREEN}Time per game: {DEFAULT}{time_per_game:.2}s.");
            println!("In total: {tot_games} [{percentage:.2}%] games are done.");
            println!("White wins: {ww}, Black wins: {bw}, Draws: {dr}");
            println!("Elapsed time: {elapsed:.2}s. {RED}ETR: {etr:.2}s{DEFAULT}");

            stdout().flush().unwrap();
        }

        // Reset everything from previous game
        output_buffer.flush().unwrap();
        board = Board::default();
        search.game_reset();
        hashes = vec![];

        // Play a new game
        // First we get a 'random' starting position
        let random_moves = rng.usize(11..=12);
        for _ in 0..random_moves {
            let moves = movegen::pure_moves(&board);
            let mv = moves[rng.usize(..moves.len())];
            board.play_unchecked(mv);

            // ... make sure that the position isn't over
            if board.status() != GameStatus::Ongoing {
                continue 'main;
            }
        }
        search.nnue.refresh(&board);

        // ... make sure that the exit isn't absurd
        let (score, _) = search.data_search(&board, SearchType::Depth(8));
        if score.abs() > 1000 {
            continue 'main;
        }

        // ... play the rest of the game
        let (game_result, winner) = loop {
            let status = board.status();
            if draw(&board, &mut hashes) {
                break (GameStatus::Drawn, None);
            }
            if status != GameStatus::Ongoing {
                break (status, Some(!board.side_to_move()));
            }

            search.go_reset();
            let (mut score, best_move) = search.data_search(&board, options.st);

            // filter noisy positions
            let not_in_check = board.checkers().is_empty();
            let okay_score = score.abs() < definitions::TB_WIN_IN_PLY;
            let okay_move = is_quiet(&board, best_move);
            if not_in_check && okay_score && okay_move {
                // Always report scores from white's perspective
                score = if board.side_to_move() == Color::White { score } else { -score };

                game_buffer.push((score, format!("{}", board)));
            }

            board.play_unchecked(best_move);
            search.nnue.refresh(&board);
        };

        // Always report wins from white's perspective
        let result_output = match (game_result, winner) {
            (GameStatus::Drawn, _) => {
                DRAWS.fetch_add(1, Ordering::Relaxed);
                "0.5".to_string()
            }
            (GameStatus::Won, Some(Color::White)) => {
                WHITE_WINS.fetch_add(1, Ordering::Relaxed);
                "1".to_string()
            }
            (GameStatus::Won, Some(Color::Black)) => {
                BLACK_WINS.fetch_add(1, Ordering::Relaxed);
                "0".to_string()
            }
            _ => unreachable!(),
        };

        // Write the result
        FENS.fetch_add(game_buffer.len() as u64, Ordering::Relaxed);
        for (score, fen) in game_buffer.drain(..) {
            writeln!(output_buffer, "{fen} | {score} | {result_output}").unwrap();
        }

        // Safely abort with CTRLC handler since otherwise
        // our files could get truncated and the data get lost.
        if STOP_FLAG.load(Ordering::SeqCst) {
            break 'main;
        }
    }

    fn draw(board: &Board, hashes: &mut Vec<u64>) -> bool {
        // Material draw
        if board.occupied().len() == 2
            || (board.occupied().len() == 3
                && !(board.pieces(Piece::Bishop) | board.pieces(Piece::Knight)).is_empty())
        {
            return true;
        }

        // 3-fold repetition
        let hash = board.hash();
        hashes.push(hash);
        if hashes.iter().filter(|&&h| h == hash).count() >= 3 {
            return true;
        }

        // 50-move rule is handled by board.status()
        false
    }
}
