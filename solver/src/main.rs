extern crate enigma;
extern crate argparse;

use std::fs;
use std::collections::{HashMap, BinaryHeap};

use enigma::Enigma;
use argparse::{ArgumentParser, Store, StoreTrue};
use std::cmp::Ordering;
use std::time::SystemTime;

const INIT_POSITIONS: [u8;3] = [1u8, 1u8, 1u8];
const STORE_TOP_RESULTS: usize = 5;
const DEFAULT_REFLECTOR: char = 'B';

#[derive(Eq)]
struct EnigmaConfig {
    rotors: [u8;3],
    settings: [u8;3],
    mappings: HashMap<char, char>,
    score: i32
}

impl Ord for EnigmaConfig {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.cmp(&self.score)
    }
}

impl PartialOrd for EnigmaConfig {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for EnigmaConfig {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

struct EnigmaResults {
    top_enigma_configs: BinaryHeap<EnigmaConfig>,
    top_n: usize
}

impl EnigmaResults {
    fn add(&mut self, res: EnigmaConfig, silent: bool) {
        let score = res.score;
        self.top_enigma_configs.push(res);


        if self.top_enigma_configs.len() > self.top_n {
            let x = self.top_enigma_configs.pop();
            match x {
                Some(x) => {
                    if x.score != score && !silent {
                        println!("New top score {}", score);
                    }
                }
                _ => {}
            }
        }
    }

    fn new(store_top_n: usize) -> EnigmaResults {
        EnigmaResults {
            top_enigma_configs: BinaryHeap::new(),
            top_n: store_top_n
        }
    }
}

// Returns the best guess for deciphering the given ciphertext.
// One simplifying assumption is that the starting position of the rotors is always 1, 1, 1.
// In real Enigma, starting position was encoded with a different cipher and sent just
// before the actual message. You had to decipher that first before deciphering the message
// itself (so technically, by this time, you should know what it is).
//
// https://en.wikipedia.org/wiki/Enigma_machine#Indicator
fn decipher(ciphertext: &str, number_of_rotors: u8, number_of_wires: u8, silent: bool) {
    let possible_rotor_configs = rotor_configs(number_of_rotors);
    let possible_rotor_settings = rotor_settings();

    println!("Initial IOC: {}", ioc(ciphertext));

    let mut res = EnigmaResults::new(STORE_TOP_RESULTS);

    // First we will zero in on rings and settings.
    // This is 336 * ~17k operations.
    for candidate_config in &possible_rotor_configs {
        if !silent {
            println!("Trying rotors [{}, {}, {}]", candidate_config[0], candidate_config[1], candidate_config[2]);
        }

        for candidate_setting in &possible_rotor_settings {
            let rotors = candidate_config.clone();
            let settings = candidate_setting.clone();

            // No substitutions to try so far
            let mappings = HashMap::new();

            let candidate_ans = cipher_string(ciphertext, rotors, settings, &mappings);
            let candidate_ioc = ioc(&candidate_ans);

            res.add(EnigmaConfig {
                rotors: candidate_config.clone(),
                settings: candidate_setting.clone(),
                mappings: HashMap::new(),
                score: candidate_ioc,
            }, silent);
        }
    }

    let top_results = res.top_enigma_configs.into_sorted_vec();

    if !silent {
        println!("Best parameters so far: rotors: {:?}, settings: {:?}, score: {}", top_results[0].rotors, top_results[0].settings, top_results[0].score);
        println!("Best guess so far: {}", cipher_string(ciphertext, top_results[0].rotors, top_results[0].settings, &top_results[0].mappings));

        println!("Deciphering substitutions");
    }

    let mut results_after_plugboards: Vec<EnigmaConfig> = vec![];
    for top_result in top_results {
        results_after_plugboards.push(solve_plugboard(top_result, ciphertext, number_of_wires, silent));
    }

    results_after_plugboards.sort();
    let top = &results_after_plugboards[0];

    println!("Best parameters: rotors: {:?}, settings: {:?}, score: {}, plugboard: {:?}", top.rotors, top.settings, top.score, top.mappings);
    println!("Best guess: {}", cipher_string(ciphertext, top.rotors, top.settings, &top.mappings));
}

// TODO: We may want to send a vector of top n instead of top 1.
fn solve_plugboard(conf: EnigmaConfig, ciphertext: &str, number_of_wires: u8, silent: bool) -> EnigmaConfig {
    //let txt = cipher_string(ciphertext, conf.rotors, conf.settings, &conf.mappings);

    // We will progressively add one mapping at a time. Add up to 10 configs until we hit 1600.
    let mut best_score = conf.score;
    let mut best_guess = conf;

    for _i in 0u8..number_of_wires {
        let x = add_best_plugwire(ciphertext, &best_guess, silent);
        if x.score > 1600 {
            return x;
        } else if x.score > best_score {
            best_score = x.score;
            best_guess = x;
        }
    }

    best_guess
}

fn add_best_plugwire(ciphertext: &str, conf: &EnigmaConfig, silent: bool) -> EnigmaConfig {
    let mut max_score = conf.score;
    let mut best_conf: Option<EnigmaConfig> = None;

    for i in 0u8..26 {
        let char1 = (i+65) as char;

        // TODO: This is really slow :(
        // Do refactor the data structures here.
        let mut mapping_exists = false;
        for (k, v) in &conf.mappings {
            if k.eq(&char1) || v.eq(&char1) {
                mapping_exists = true;
            }
        }

        if mapping_exists {
            continue;
        }

        for j in i+1..26 {
            let char2 = (j+65) as char;

            let mut mapping_exists = false;
            for (k, v) in &conf.mappings {
                if k.eq(&char2) || v.eq(&char2) {
                    mapping_exists = true;
                }
            }

            if mapping_exists {
                continue;
            }

            let mut new_mapping = conf.mappings.clone();

            new_mapping.insert(char1, char2);

            let decoded = cipher_string(ciphertext, conf.rotors, conf.settings, &new_mapping);
            let score = ioc(&decoded);
            if score > max_score {
                if !silent {
                    println!("New max score {} after [{}:{}]", score, (i+65) as char, (j+65) as char);
                }

                max_score = score;
                best_conf = Some(EnigmaConfig {
                    rotors: conf.rotors,
                    settings: conf.settings,
                    mappings: new_mapping,
                    score
                });
            }
        }
    }

    best_conf.unwrap()
}

fn cipher_string(string: &str, rotors: [u8;3], settings: [u8;3], mappings: &HashMap<char, char>) -> String {
    let mut enigma = Enigma::new_enigma(rotors, INIT_POSITIONS, settings, &mappings, DEFAULT_REFLECTOR);

    let mut s = String::new();
    for c in string.chars() {
        s.push(enigma.encrypt(c));
    }

    return s
}

// Index of coincidence will be our objective function.
// https://en.wikipedia.org/wiki/Index_of_coincidence
fn ioc(text: &str) -> i32 {
    let n: f32 = text.len() as f32;
    let c: f32 = 26.;

    let mut hist: [i32;26] = [0;26];
    for c in text.chars() {
        let i = c as usize - 65;
        hist[i] += 1;
    }

    let mut numerator: i32 = 0;
    for i in 0..26 {
        numerator += hist[i] * (hist[i] - 1);
    }

    let denominator = (n * (n-1.)) / c;

    ((numerator as f32 / denominator) * 1000f32) as i32
}

// Pre-generates all configurations for ring settings.
// There are 26 total positions to choose three from with repeats allowed.
fn rotor_settings() -> Vec<[u8;3]> {
    n_choose_three(26, true, true)
}

// Pre-generates all configurations for rotors.
// There are 8 rotors to choose 3 from with no repeats.
fn rotor_configs(number_of_rotors: u8) -> Vec<[u8;3]> {
    n_choose_three(number_of_rotors, false, false)
}

// General function to choose 3 from n with or without replacement.
fn n_choose_three(n: u8, repeat: bool, base_one: bool) -> Vec<[u8;3]> {
    let mut configs : Vec<[u8;3]> = vec![];

    for i in 0u8..n {
        for j in 0u8..n {
            if !repeat && i == j { continue; }
            for k in 0u8..n {
                if !repeat && (k == j || k == i) { continue; }
                if base_one {
                    configs.push([i+1, j+1, k+1]);
                } else {
                    configs.push([i, j, k]);
                }
            }
        }
    }

    return configs;
}

fn main() {
    let mut silent = false;
    let mut number_of_rotors = 8u8;
    let mut number_of_wires = 0u8;
    let mut ciphertext_file = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut ciphertext_file)
            .add_option(&["-c", "--ciphertext-file"], Store, "Ciphertext");
        ap.refer(&mut number_of_rotors)
            .add_option(&["-r", "--rotors"], Store, "Number of rotors");
        ap.refer(&mut number_of_wires)
            .add_option(&["-w", "--wires"], Store, "Number of wires");
        ap.refer(&mut silent)
            .add_option(&["-s", "--silent"], StoreTrue, "Log fewer things");
        ap.parse_args_or_exit();
    }

    let text = fs::read_to_string(ciphertext_file)
        .expect("Could not read file");

    let mut preproc_text = String::new();
    for c in text.chars() {
        let cu8: u8 = c as u8;
        if (cu8 >= 65 && cu8 <= 90) || (cu8 >= 97 && cu8 <= 122) {
            preproc_text.push(c.to_ascii_uppercase());
        }
    }

    println!("Input text: {}", preproc_text);

    let timer = SystemTime::now();
    decipher(&preproc_text, number_of_rotors, number_of_wires, silent);
    println!("Deciphered in {}ms", timer.elapsed().unwrap().as_millis());
}
