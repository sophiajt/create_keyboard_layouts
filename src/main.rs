use std::{
    collections::HashMap,
    fmt::Display,
    fs::OpenOptions,
    io::{Read, Write},
    sync::mpsc::channel,
    thread,
};

use iter_tools::prelude::*;
use rand::{seq::SliceRandom, thread_rng, RngCore};

const ALGORITHM_VERSION: f64 = 4.419;
const MAX_SAMPLES_PER_CATEGORY: usize = 1000;
const FAILED_TO_IMPROVE_LIMIT: usize = 1000;

const SINGLE_TOP: i64 = 0;
const SINGLE_MIDDLE: i64 = 3;
const SINGLE_BOTTOM: i64 = 0;

const DOUBLE_TOP_INWARD: i64 = 5;
const DOUBLE_TOP_OUTWARD: i64 = 3;
const DOUBLE_MIDDLE_INWARD: i64 = 10;
const DOUBLE_MIDDLE_OUTWARD: i64 = 5;

const TRIPLE_TOP_INWARD: i64 = 35;
const TRIPLE_TOP_OUTWARD: i64 = 25;
const TRIPLE_MIDDLE_INWARD: i64 = 35;
const TRIPLE_MIDDLE_OUTWARD: i64 = 25;

const JROLL_INWARD: i64 = 25;
const JROLL_OUTWARD: i64 = 10;

const QUADRUPLE_TOP_INWARD: i64 = 20;
const QUADRUPLE_TOP_OUTWARD: i64 = 20;
const QUADRUPLE_MIDDLE_INWARD: i64 = 20;
const QUADRUPLE_MIDDLE_OUTWARD: i64 = 20;

const LONG_JROLL_INWARD: i64 = 20;

const CENTER_TOP_PENALTY: i64 = -10;
const CENTER_MIDDLE_PENALTY: i64 = -10;
const CENTER_BOTTOM_PENALTY: i64 = -10;

const MINOR_FINGER_CURL_PENALTY: i64 = -5;
const PINKIE_PENALTY: i64 = -5;
const TWO_ROW_MOVE_PENALTY: i64 = -10;
const SAME_FINGER_PENALTY: i64 = -5;

const USE_QUADRUPLE_ROLL: bool = false;

// This will prevent using the ,./ keys
const BOTTOM_RIGHT_PENALTY: i64 = -10000000;
// const BOTTOM_RIGHT_PENALTY: i64 = 0;

#[derive(Clone)]
struct Keyboard {
    name: String,
    rows: Vec<Vec<u8>>,
}

impl Display for Keyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        for row in &self.rows {
            for col in row {
                let _ = f.write_char(*col as char);
            }
        }
        Ok(())
    }
}
impl Keyboard {
    pub fn qwerty() -> Keyboard {
        Keyboard {
            name: "qwerty".into(),
            rows: vec![
                "qwertyuiop".into(),
                "asdfghjkl;".into(),
                "zxcvbnm,./".into(),
            ],
        }
    }

    pub fn dvorak() -> Keyboard {
        Keyboard {
            name: "dvorak".into(),
            rows: vec![
                "',.pyfgcrl".into(),
                "aoeuidhtns".into(),
                ";qjkxbmwvz".into(),
            ],
        }
    }

    pub fn colemak() -> Keyboard {
        Keyboard {
            name: "colemak".into(),
            rows: vec![
                "qwfpgjluy;".into(),
                "arstdhneio".into(),
                "zxcvbkm,./".into(),
            ],
        }
    }

    pub fn workman() -> Keyboard {
        Keyboard {
            name: "workman".into(),
            rows: vec![
                "qdrwbjfup;".into(),
                "ashtgyneoi".into(),
                "zxmcvkl,./".into(),
            ],
        }
    }

    // pub fn inas() -> Keyboard {
    //     Keyboard {
    //         name: "inas".into(),
    //         rows: vec![
    //             "pmfcqxluoy".into(),
    //             "inasbkreht".into(),
    //             "_vgdzjw___".into(),
    //         ],
    //     }
    // }

    pub fn random_layout() -> Keyboard {
        let mut keys: Vec<u8> = "abcdefghijklmnopqrstuvwxyz____".into();

        keys.shuffle(&mut thread_rng());

        Keyboard {
            name: "random".into(),
            rows: vec![keys[0..10].into(), keys[10..20].into(), keys[20..30].into()],
        }
    }

    pub fn find_key(&self, key: u8) -> (usize, usize) {
        for row in 0..3 {
            for col in 0..10 {
                if self.rows[row][col] == key {
                    return (row, col);
                }
            }
        }
        panic!("Key not on keyboard")
    }
}

#[derive(Clone)]
struct Scorer {
    single_byte: Vec<(u8, i64)>,
    double_byte: Vec<(Vec<u8>, i64)>,
    triple_byte: Vec<(Vec<u8>, i64)>,
    quadruple_byte: Vec<(Vec<u8>, i64)>,
}

impl Scorer {
    fn score_singles(&self, kb: &Keyboard) -> i64 {
        let mut total = 0;

        for byte in &self.single_byte {
            if kb.rows[0].contains(&byte.0) {
                total += byte.1 * SINGLE_TOP;
            } else if kb.rows[1].contains(&byte.0) {
                total += byte.1 * SINGLE_MIDDLE;
            } else if kb.rows[2].contains(&byte.0) {
                total += byte.1 * SINGLE_BOTTOM;
            }
        }
        total
    }

    fn score_doubles(&self, kb: &Keyboard) -> i64 {
        let mut total = 0;

        'top: for bytes in &self.double_byte {
            let forward = (bytes.0[0], bytes.0[1]);
            let backward = (bytes.0[1], bytes.0[0]);

            // Left hand, middle row
            for (a, b) in kb.rows[1][0..4].iter().tuple_windows() {
                let value = (*a, *b);

                if value == forward {
                    total += bytes.1 * DOUBLE_MIDDLE_INWARD;
                    continue 'top;
                }
                if value == backward {
                    total += bytes.1 * DOUBLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }

            // Right hand, middle row
            for (a, b) in kb.rows[1][6..].iter().tuple_windows() {
                let value = (*a, *b);

                if value == backward {
                    total += bytes.1 * DOUBLE_MIDDLE_INWARD;
                    continue 'top;
                }
                if value == forward {
                    total += bytes.1 * DOUBLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }

            // Left hand, top row
            for (a, b) in kb.rows[0][0..4].iter().tuple_windows() {
                let value = (*a, *b);

                if value == forward {
                    total += bytes.1 * DOUBLE_TOP_INWARD;
                    continue 'top;
                }
                if value == backward {
                    total += bytes.1 * DOUBLE_TOP_OUTWARD;
                    continue 'top;
                }
            }

            // Right hand, top row
            for (a, b) in kb.rows[0][6..].iter().tuple_windows() {
                let value = (*a, *b);

                if value == backward {
                    total += bytes.1 * DOUBLE_MIDDLE_INWARD;
                    continue 'top;
                }
                if value == forward {
                    total += bytes.1 * DOUBLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }
        }
        total
    }

    fn score_triples(&self, kb: &Keyboard) -> i64 {
        let mut total = 0;

        'top: for bytes in &self.triple_byte {
            let forward = (bytes.0[0], bytes.0[1], bytes.0[2]);
            let backward = (bytes.0[2], bytes.0[1], bytes.0[0]);

            // Left hand, middle row
            for (a, b, c) in kb.rows[1][0..4].iter().tuple_windows() {
                let value = (*a, *b, *c);

                if value == forward {
                    total += bytes.1 * TRIPLE_MIDDLE_INWARD;
                    continue 'top;
                }

                if value == backward {
                    total += bytes.1 * TRIPLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }

            // Right hand, middle row
            for (a, b, c) in kb.rows[1][6..].iter().tuple_windows() {
                let value = (*a, *b, *c);

                if value == backward {
                    total += bytes.1 * TRIPLE_MIDDLE_INWARD;
                    continue 'top;
                }
                if value == forward {
                    total += bytes.1 * TRIPLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }

            // Left hand, top row
            for (a, b, c) in kb.rows[0][0..4].iter().tuple_windows() {
                let value = (*a, *b, *c);

                if value == forward {
                    total += bytes.1 * TRIPLE_TOP_INWARD;
                    continue 'top;
                }
                if value == backward {
                    total += bytes.1 * TRIPLE_TOP_OUTWARD;
                    continue 'top;
                }
            }

            // Right hand, top row
            for (a, b, c) in kb.rows[0][6..].iter().tuple_windows() {
                let value = (*a, *b, *c);

                if value == backward {
                    total += bytes.1 * TRIPLE_MIDDLE_INWARD;
                    continue 'top;
                }
                if value == forward {
                    total += bytes.1 * TRIPLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }

            // Left hand, j-rolls
            if kb.rows[1][0] == forward.0
                && kb.rows[1][1] == forward.1
                && kb.rows[2][2] == forward.2
            {
                total += bytes.1 * JROLL_INWARD;
                continue;
            }

            if kb.rows[1][1] == forward.0
                && kb.rows[1][2] == forward.1
                && kb.rows[2][2] == forward.2
            {
                total += bytes.1 * JROLL_INWARD;
                continue;
            }

            if kb.rows[1][1] == forward.2
                && kb.rows[1][2] == forward.1
                && kb.rows[2][2] == forward.0
            {
                total += bytes.1 * JROLL_OUTWARD;
                continue;
            }

            // Right hand, j-rolls
            if kb.rows[1][9] == forward.0
                && kb.rows[1][8] == forward.1
                && kb.rows[2][6] == forward.2
            {
                total += bytes.1 * JROLL_INWARD;
                continue;
            }

            if kb.rows[1][8] == forward.0
                && kb.rows[1][7] == forward.1
                && kb.rows[2][6] == forward.2
            {
                total += bytes.1 * JROLL_INWARD;
                continue;
            }

            if kb.rows[1][8] == forward.2
                && kb.rows[1][7] == forward.1
                && kb.rows[2][6] == forward.0
            {
                total += bytes.1 * JROLL_OUTWARD;
                continue;
            }
        }
        total
    }

    fn score_quadruples(&self, kb: &Keyboard) -> i64 {
        let mut total = 0;

        'top: for bytes in &self.quadruple_byte {
            let forward = (bytes.0[0], bytes.0[1], bytes.0[2], bytes.0[3]);
            let backward = (bytes.0[3], bytes.0[2], bytes.0[1], bytes.0[0]);

            // Left hand, middle row
            for (a, b, c, d) in kb.rows[1][0..4].iter().tuple_windows() {
                let value = (*a, *b, *c, *d);

                if value == forward {
                    total += bytes.1 * QUADRUPLE_MIDDLE_INWARD;
                    continue 'top;
                }

                if value == backward {
                    total += bytes.1 * QUADRUPLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }

            // Right hand, middle row
            for (a, b, c, d) in kb.rows[1][6..].iter().tuple_windows() {
                let value = (*a, *b, *c, *d);

                if value == backward {
                    total += bytes.1 * QUADRUPLE_MIDDLE_INWARD;
                    continue 'top;
                }
                if value == forward {
                    total += bytes.1 * QUADRUPLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }

            // Left hand, top row
            for (a, b, c, d) in kb.rows[0][0..4].iter().tuple_windows() {
                let value = (*a, *b, *c, *d);

                if value == forward {
                    total += bytes.1 * QUADRUPLE_TOP_INWARD;
                    continue 'top;
                }
                if value == backward {
                    total += bytes.1 * QUADRUPLE_TOP_OUTWARD;
                    continue 'top;
                }
            }

            // Right hand, top row
            for (a, b, c, d) in kb.rows[0][6..].iter().tuple_windows() {
                let value = (*a, *b, *c, *d);

                if value == backward {
                    total += bytes.1 * QUADRUPLE_MIDDLE_INWARD;
                    continue 'top;
                }
                if value == forward {
                    total += bytes.1 * QUADRUPLE_MIDDLE_OUTWARD;
                    continue 'top;
                }
            }

            if kb.rows[1][1] == forward.0
                && kb.rows[1][2] == forward.1
                && kb.rows[1][3] == forward.2
                && kb.rows[2][3] == forward.3
            {
                total += bytes.1 * LONG_JROLL_INWARD;
                continue;
            }
            if kb.rows[1][9] == forward.0
                && kb.rows[1][8] == forward.1
                && kb.rows[1][7] == forward.2
                && kb.rows[2][6] == forward.3
            {
                total += bytes.1 * LONG_JROLL_INWARD;
                continue;
            }
        }
        total
    }

    fn score_penalties(&self, kb: &Keyboard) -> i64 {
        let mut total: i64 = 0;

        for byte in &self.single_byte {
            if kb.rows[0][4] == byte.0 || kb.rows[0][5] == byte.0 {
                total += byte.1 * CENTER_TOP_PENALTY;
            }
            if kb.rows[1][4] == byte.0 || kb.rows[1][5] == byte.0 {
                total += byte.1 * CENTER_MIDDLE_PENALTY;
            }
            if kb.rows[2][4] == byte.0 || kb.rows[2][5] == byte.0 {
                total += byte.1 * CENTER_BOTTOM_PENALTY;
            }
            if kb.rows[0][0] == byte.0
                || kb.rows[2][0] == byte.0
                || kb.rows[2][9] == byte.0
                || kb.rows[0][9] == byte.0
            {
                total += byte.1 * PINKIE_PENALTY;
            }
            if kb.rows[2][0] == byte.0
                || kb.rows[2][1] == byte.0
                || kb.rows[2][9] == byte.0
                || kb.rows[2][8] == byte.0
            {
                total += byte.1 * MINOR_FINGER_CURL_PENALTY;
            }

            // Protect the bottom three keys so we can use what is usually there
            // This isn't strictly necessary but helps with adapting the layout
            if kb.rows[2][7] == byte.0 || kb.rows[2][8] == byte.0 || kb.rows[2][9] == byte.0 {
                total += BOTTOM_RIGHT_PENALTY
            }
        }

        // Penalty for jumping between top and bottom rows
        for double in &self.double_byte {
            let (from_row, from_col) = kb.find_key(double.0[0]);
            let (to_row, to_col) = kb.find_key(double.0[1]);

            if (from_row == 0 && to_row == 2) || (from_row == 2 && to_row == 0) {
                total += double.1 * TWO_ROW_MOVE_PENALTY;
            }

            if from_col == to_col && from_row != to_row {
                total += double.1 * SAME_FINGER_PENALTY;
            }
        }

        total
    }

    pub fn score_keyboard(&self, kb: &Keyboard) -> i64 {
        let mut total: i64 = 0;

        total += self.score_singles(kb);
        total += self.score_doubles(kb);
        total += self.score_triples(kb);
        if USE_QUADRUPLE_ROLL {
            total += self.score_quadruples(kb);
        }

        total += self.score_penalties(kb);

        total
    }

    pub fn debug(&self) {
        for (k, v) in &self.single_byte {
            println!("{}: {}", *k as char, v);
        }

        for (k, v) in &self.double_byte {
            println!("{}{}: {}", k[0] as char, k[1] as char, v);
        }

        for (k, v) in &self.triple_byte {
            println!("{}{}{}: {}", k[0] as char, k[1] as char, k[2] as char, v);
        }

        for (k, v) in &self.quadruple_byte {
            println!(
                "{}{}{}{}: {}",
                k[0] as char, k[1] as char, k[2] as char, k[3] as char, v
            );
        }
    }
}

fn random_swap(kb: &mut Keyboard) {
    let from_row = (rand::thread_rng().next_u64() % 3) as usize;
    let from_col = (rand::thread_rng().next_u64() % 10) as usize;

    let to_row = (rand::thread_rng().next_u64() % 3) as usize;
    let to_col = (rand::thread_rng().next_u64() % 10) as usize;

    let prev = kb.rows[to_row][to_col];
    kb.rows[to_row][to_col] = kb.rows[from_row][from_col];
    kb.rows[from_row][from_col] = prev;
}

fn find_keyboard(scorer: &Scorer) -> (i64, Keyboard) {
    let mut keyboard = Keyboard::random_layout();

    let mut current_score = scorer.score_keyboard(&keyboard);
    let mut time_since_last_improvement = 0;

    loop {
        time_since_last_improvement += 1;

        let mut new_keyboard = keyboard.clone();

        random_swap(&mut new_keyboard);

        let new_score = scorer.score_keyboard(&new_keyboard);

        if new_score > current_score {
            keyboard = new_keyboard;
            current_score = new_score;
            time_since_last_improvement = 0;
        }

        if time_since_last_improvement >= FAILED_TO_IMPROVE_LIMIT {
            break;
        }
    }

    (current_score, keyboard)
}

fn main() {
    let mut input = vec![];
    let mut debug = false;

    println!("Loading in corpus...");
    for file in std::env::args().skip(1) {
        if file == "--debug" {
            debug = true;
            continue;
        }
        let file = std::fs::File::open(file).unwrap();

        let mut handle = file.take(100 * 1024 * 1024);

        let mut buf = vec![0u8; 100 * 1024 * 1024];
        let _ = handle.read(&mut buf).unwrap();

        // let mut buf = buf.to_ascii_lowercase();

        input.append(&mut buf);
    }

    // Work around spaces
    // let input: Vec<_> = input.into_iter().filter(|x| *x != b' ').collect();

    println!("Counting singles...");
    // Single byte counts
    let mut single_byte: HashMap<u8, i64> = HashMap::new();
    for a in input.iter() {
        if (b'a'..=b'z').contains(a) {
            *single_byte.entry(*a).or_default() += 1;
        }
    }

    println!("Counting doubles...");
    // Double byte counts
    let mut double_byte: HashMap<Vec<u8>, i64> = HashMap::new();
    for (a, b) in input.iter().tuple_windows() {
        if (b'a'..=b'z').contains(a) && (b'a'..=b'z').contains(b) && a != b {
            let item = vec![*a, *b];
            *double_byte.entry(item).or_default() += 1;
        }
    }

    println!("Counting triples...");
    // Triple byte counts
    let mut triple_byte: HashMap<Vec<u8>, i64> = HashMap::new();
    for (a, b, c) in input.iter().tuple_windows() {
        if (b'a'..=b'z').contains(a)
            && (b'a'..=b'z').contains(b)
            && (b'a'..=b'z').contains(c)
            && ![b, c].contains(&a)
            && b != c
        {
            let item = vec![*a, *b, *c];

            *triple_byte.entry(item).or_default() += 1;
        }
    }

    println!("Counting quadruples...");
    // Quadruple byte counts
    let mut quadruple_byte: HashMap<Vec<u8>, i64> = HashMap::new();
    for (a, b, c, d) in input.iter().tuple_windows() {
        if (b'a'..=b'z').contains(a)
            && (b'a'..=b'z').contains(b)
            && (b'a'..=b'z').contains(c)
            && (b'a'..=b'z').contains(d)
            && ![b, c, d].contains(&a)
            && ![c, d].contains(&b)
            && c != d
        {
            let item = vec![*a, *b, *c, *d];

            *quadruple_byte.entry(item).or_default() += 1;
        }
    }

    let single_byte: Vec<_> = single_byte
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
        .take(MAX_SAMPLES_PER_CATEGORY)
        .collect();

    let double_byte: Vec<_> = double_byte
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
        .take(MAX_SAMPLES_PER_CATEGORY)
        .collect();

    let triple_byte: Vec<_> = triple_byte
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
        .take(MAX_SAMPLES_PER_CATEGORY)
        .collect();

    let quadruple_byte: Vec<_> = quadruple_byte
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
        .take(MAX_SAMPLES_PER_CATEGORY)
        .collect();

    let scorer = Scorer {
        single_byte,
        double_byte,
        triple_byte,
        quadruple_byte,
    };

    if debug {
        scorer.debug();
        return;
    }

    let standard_keyboards = vec![
        Keyboard::qwerty(),
        Keyboard::dvorak(),
        Keyboard::colemak(),
        Keyboard::workman(),
    ];

    // Show the score for the standard keyboards
    // for this round of scoring
    let mut dictionary = HashMap::new();
    for kb in &standard_keyboards {
        let score = scorer.score_keyboard(kb);

        dictionary.insert(kb.name.clone(), score);
    }

    let mut result: Vec<_> = dictionary.into_iter().collect();

    result.sort_by(|a, b| (a.1).partial_cmp(&b.1).unwrap());

    println!("algorithm: {}", ALGORITHM_VERSION);
    for keyboard in result {
        println!("{}: {}", keyboard.0, keyboard.1);
    }

    // println!("Finding a keyboard...");
    // let kb = Keyboard::jt();
    // // let (kb, score) = find_keyboard(&scorer);
    // let score = scorer.score_keyboard(&kb);
    // println!("{}|{}", score, kb);

    let (sender, receiver) = channel();

    for _ in 1..10 {
        let sender = sender.clone();
        let scorer = scorer.clone();

        thread::spawn(move || loop {
            let (score, kb) = find_keyboard(&scorer);

            let _ = sender.send((score, kb));
        });
    }

    let mut best = i64::MIN;

    loop {
        let (score, kb) = receiver.recv().unwrap();

        if score > best {
            println!("New best: {}|{}|{}", ALGORITHM_VERSION, score, kb);
            best = score;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("output.log")
            .unwrap();

        if let Err(e) = writeln!(file, "{}|{}|{}", ALGORITHM_VERSION, score, kb) {
            eprintln!("Couldn't write to file: {}", e);
        }
        print!(".");
        let _ = std::io::stdout().flush();
    }
}
