use std::{
    fs::File,
    io::{stdin, BufWriter, Write},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const DEFAULT: &str = "\x1b[0m";
const WHITE: &str = "\x1b[38;5;15m";
const ORANGE: &str = "\x1b[38;5;208m";
const GREEN: &str = "\x1b[38;5;40m";
const RED: &str = "\x1b[38;5;196m";

static FENS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
enum SearchType {
    Depth(i32),
    Nodes(u64),
}

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

pub fn root() {
    // Get the number of games to generate
    println!("How many games would you like to gen? [1, 100M]");
    let mut inp_games = String::new();
    stdin().read_line(&mut inp_games).unwrap();
    let games = match inp_games.trim().parse::<usize>() {
        Ok(games) => games,
        Err(games) => {
            panic!("Invalid games input! {games}")
        }
    };
    if !(1..=100_000_000).contains(&games) {
        panic!("Invalid game range! {games}, needs to be between [1, 100M]")
    }

    // Get the number of threads to use
    println!("How many threads would you like to use? [1, 64]");
    let mut inp_threads = String::new();
    stdin().read_line(&mut inp_threads).unwrap();
    let threads = match inp_threads.trim().parse::<usize>() {
        Ok(threads) => threads,
        Err(threads) => {
            panic!("Invalid threads input! {threads}")
        }
    };
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
            let depth = match inp_depth.trim().parse::<i32>() {
                Ok(depth) => depth,
                Err(depth) => {
                    panic!("Invalid depth input! {depth}")
                }
            };
            if !(1..=100).contains(&depth) {
                panic!("Invalid depth range! {depth}, needs to be between [1, 100]")
            }
            SearchType::Depth(depth)
        }
        "nodes" => {
            println!("What nodes would you like to use? [1, 100M]");
            let mut inp_nodes = String::new();
            stdin().read_line(&mut inp_nodes).unwrap();
            let nodes = match inp_nodes.trim().parse::<u64>() {
                Ok(nodes) => nodes,
                Err(nodes) => {
                    panic!("Invalid nodes input! {nodes}")
                }
            };
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
}

fn generate_main(params: Parameters) {
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

fn generate_thread(id: usize, data_dir: &PathBuf, options: &Parameters) {
    let mut output_file = File::create(data_dir.join(format!("thread_{id}.txt"))).unwrap();
    let mut output_buffer = BufWriter::new(&mut output_file);
}
