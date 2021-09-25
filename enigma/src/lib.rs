mod helpers;

use helpers::helpers::*;
use std::collections::HashMap;

const CHARSET_SIZE: u8 = 26;
const ILLEGAL_LOC: u8 = 27;

struct RotorConfig {
    encoding: &'static str,
    notches: [u8;2]
}

const ROTOR_CONFIGS: [RotorConfig;8] = [
    RotorConfig { encoding: "EKMFLGDQVZNTOWYHXUSPAIBRCJ", notches: [ILLEGAL_LOC, 16] },
    RotorConfig { encoding: "AJDKSIRUXBLHWTMCQGZNPYFVOE", notches: [ILLEGAL_LOC, 4] },
    RotorConfig { encoding: "BDFHJLCPRTXVZNYEIWGAKMUSQO", notches: [ILLEGAL_LOC, 21] },
    RotorConfig { encoding: "ESOVPZJAYQUIRHXLNFTGKDCMWB", notches: [ILLEGAL_LOC, 9] },

    RotorConfig { encoding: "VZBRGITYUPSDNHLXAWMJQOFECK", notches: [ILLEGAL_LOC, 25] },
    RotorConfig { encoding: "JPGVOUMFYQBENHZRDKASXLICTW", notches: [12, 25] },
    RotorConfig { encoding: "NZJHGRCXMYSWBOUFAIVLPEKQDT", notches: [12, 25] },
    RotorConfig { encoding: "FKQHTLXOCBJSPDZRAMEWNIUYGV", notches: [12, 25] },
];

struct Rotor {
    notches: [u8;2],
    position: u8,
    setting: u8,

    forward_mapping: [u8;26],
    reverse_mapping: [u8;26]
}

impl Rotor {
    fn at_notch(&self) -> bool {
        self.notches[0] == self.position || self.notches[1] == self.position
    }

    fn turnover(&mut self) {
        self.position = (self.position + 1) % CHARSET_SIZE;
    }

    fn forward(&self, num: u8) ->  u8 {
        let shift_add: u8 = 26 + self.position - self.setting;
        let shift_sub: u8 = 26 - self.position + self.setting;

        let x: u8 = (num + shift_add) % 26;
        ((self.forward_mapping[x as usize] + shift_sub) % 26) as u8
    }

    fn backward(&self, num: u8) -> u8 {
        let shift_add: u8 = 26 + self.position - self.setting;
        let shift_sub: u8 = 26 - self.position + self.setting;

        let x: u8 = (num + shift_add) % 26;
        ((self.reverse_mapping[x as usize] + shift_sub) % 26) as u8
    }

    fn rotor_by_index(idx: u8, position: u8, setting: u8) -> Rotor {
        let rotor_config: &RotorConfig = &ROTOR_CONFIGS[idx as usize];
        let enc_u8: [u8;26] = c2narray_arr26(&rotor_config.encoding);
        let rev_enc_u8: [u8;26] = inv_mapping_arr26(enc_u8);

        Rotor {
            notches: rotor_config.notches,
            position,
            setting,
            forward_mapping: enc_u8,
            reverse_mapping: rev_enc_u8
        }
    }
}

struct Reflector {
    reflector_mapping_arr: [u8;26]
}

impl Reflector {

    fn forward(&self, num: u8) -> u8 {
        self.reflector_mapping_arr[num as usize]
    }

    fn reflection_from_letterid(letterid:char) -> Reflector {
        Reflector {
            reflector_mapping_arr: match letterid {
                'B' => c2narray_arr26("YRUHQSLDPXNGOKMIEBFZCWVJAT"),
                'C' => c2narray_arr26("FVPJIAOYEDRZXWGCTKUQSBNMHL"),
                _ => c2narray_arr26("ZYXWVUTSRQPONMLKJIHGFEDCBA")
            }
        }
    }
}

struct Plugboard {
    plugboard_mappings: [u8;26]
}

impl Plugboard {
    fn forward(&self, num: u8) -> u8 {
        self.plugboard_mappings[num as usize]
    }

    fn new_plugboard(letter_mapping: &HashMap<char, char>) -> Plugboard {
        let mut mappings: [u8;26] = [0;26];

        for i in 0..26 {
            mappings[i] = i as u8;
        }

        for (key, value) in letter_mapping {
            mappings[c2n(*key) as usize] = c2n(*value);
            mappings[c2n(*value) as usize] = c2n(*key);
        }

        Plugboard {
            plugboard_mappings: mappings
        }
    }
}

pub struct Enigma {
    plugboard: Plugboard,
    left_rotor: Rotor,
    middle_rotor: Rotor,
    right_rotor: Rotor,
    reflector: Reflector
}

impl Enigma {
    /**
     * Rotate the rotors that need rotating.
     * https://en.wikipedia.org/wiki/Enigma_machine#Turnover
     *
     * Reference implementation that helped me "decipher" the wikipedia description
     * https://github.com/mikepound/enigma/blob/main/src/com/mikepound/enigma/Enigma.java
     */
    fn rotate(&mut self) {
        // Middle rotor causes double stepping
        if self.middle_rotor.at_notch() {
            self.middle_rotor.turnover();
            self.left_rotor.turnover();
        }

        if self.right_rotor.at_notch() {
            self.middle_rotor.turnover();
        }

        // Right-most one always rotates
        self.right_rotor.turnover();
    }

    pub fn encrypt(&mut self, c: char) -> char {
        let num = c2n(c);

        self.rotate();

        let num = self.plugboard.forward(num);

        let num = self.right_rotor.forward(num);
        let num = self.middle_rotor.forward(num);
        let num = self.left_rotor.forward(num);

        let num = self.reflector.forward(num);

        let num = self.left_rotor.backward(num);
        let num = self.middle_rotor.backward(num);
        let num = self.right_rotor.backward(num);

        let num = self.plugboard.forward(num);

        return n2c(num);
    }

    pub fn new_enigma(rotor_indexes: [u8;3],
                      rotor_positions: [u8;3],
                      rotor_settings: [u8;3],
                      plugboard_mappings: &HashMap<char, char>,
                      reflector_letterid: char) -> Enigma {
        Enigma {
            plugboard: Plugboard::new_plugboard(plugboard_mappings),
            left_rotor: Rotor::rotor_by_index(rotor_indexes[0], rotor_positions[0], rotor_settings[0]),
            middle_rotor: Rotor::rotor_by_index(rotor_indexes[1], rotor_positions[1], rotor_settings[1]),
            right_rotor: Rotor::rotor_by_index(rotor_indexes[2], rotor_positions[2], rotor_settings[2]),
            reflector: Reflector::reflection_from_letterid(reflector_letterid)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::SystemTime;
    use crate::Enigma;

    #[test]
    fn benchmark_test() {
        let rotor_indexes = [7u8, 5, 3];
        let rotor_positions = [18u8, 10, 12];
        let rotor_settings = [1u8, 1, 5];
        let mut plugboard_mappings: HashMap<char, char> = HashMap::new();
        plugboard_mappings.insert('B', 'Q');
        plugboard_mappings.insert('C', 'R');
        plugboard_mappings.insert('D', 'I');
        plugboard_mappings.insert('E', 'J');
        plugboard_mappings.insert('K', 'W');
        plugboard_mappings.insert('M', 'T');
        plugboard_mappings.insert('O', 'S');
        plugboard_mappings.insert('P', 'X');
        plugboard_mappings.insert('U', 'Z');
        plugboard_mappings.insert('G', 'H');

        let mut enigma = Enigma::new_enigma(rotor_indexes,
                                            rotor_positions,
                                            rotor_settings,
                                            &plugboard_mappings,
                                            'B');

        let timer = SystemTime::now();

        for i in 0..100000 {
            let string = "HELLOWORLDANDTHISISJUSTATESTHJDLSDHGUROSLJKSHDJKSBDJKBSJKDBSKJBFJKSBFJKSFGHJGHGJYKFYJFYKTFTFKYFTFIYU";

            let mut write_to: String = String::with_capacity(string.len());
            for c in string.chars() {
                write_to.push(enigma.encrypt(c));
            }

            if i == 0 {
                assert_eq!(write_to, "OJWAHLFOZNXGNBBWWJTSSWCSHSYLZMTENWAMIMUGRTFFJMYNTQCNSJAKTUYJRDSCCOHEXERXDIGVQWAPABBBNUQMDNFJXKKOXSQM");
            }
        }

        println!("Test duration: {}ms", timer.elapsed().unwrap().as_millis());
    }
}
