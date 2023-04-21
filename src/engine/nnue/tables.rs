use tabled::{settings::Style, Table, Tabled};

#[derive(Tabled)]
struct Net {
    network: &'static str,
    training: &'static str,
    data: &'static str,
    report: &'static str,
    notes: &'static str,
}

pub fn print_net_history() {
    let nets = vec![
        Net {
            network: "svart0001",
            training: "lr 0.01 epochs 45 drop at 30 wdl 0.3",
            data: "90M D7 fens self-play from UHO_XXL book at varying plies",
            report: "equal to hce",
            notes: "how unremarkable",
        },
        Net {
            network: "svart0002",
            training: "lr 0.01 epochs 30 wdl 0.1",
            data: "-||-",
            report: "98.96 +/- 26.44 to hce",
            notes: "",
        },
        Net {
            network: "svart0003",
            training: "lr 0.01 epochs 80 drop at 30 wdl 0.3",
            data: "-||-",
            report: "-63.23 +/- 97.40 to hce",
            notes: "",
        },
        Net {
            network: "svart0004",
            training: "lr 0.01 epochs 80 drop at 30 wdl 0.1",
            data: "91M D8 fens generated internally with 12 random opening moves",
            report: "401.50 +/- 41.91 ",
            notes: "there it is!",
        },
        Net {
            network: "svart0005",
            training: "lr 0.01 epochs 45 drop at 30 wdl 0.25",
            data: "252M | 210M d8 and 40M 5kn",
            report: "109.42 +- 26.52 ",
            notes: "RL looking great",
        },
        Net {
            network: "svart0006",
            training: "lr 0.01 epochs 60 drop at 30 wdl 0.25",
            data: "410M fens | svart0005 data interleaved with 160M 5kn by Plutie",
            report: "14.33 +- 8.27 ",
            notes: "hidden layer size 256 -> 512",
        },
        Net {
            network: "svart0007",
            training: "lr 0.01 epochs 60 drop at 30 wdl 0.25",
            data: "-||-",
            report: "-0.22 +- 3.64 ",
            notes: "CReLu -> SCReLu",
        },
    ];

    let style = Style::rounded();
    let table = Table::new(nets).with(style).to_string();
    println!("{table}");
}
