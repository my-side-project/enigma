pub mod helpers {
    const BASE_CHAR: u8 = 65;

    pub fn c2n(character:char) -> u8 {
        (character as u8) - BASE_CHAR
    }

    pub fn n2c(number:u8) -> char {
        (number + BASE_CHAR) as char
    }

    pub fn c2narray_arr26(string: &str) -> [u8;26] {
        let mut output: [u8;26] = [0;26];
    
        let mut i = 0;
        for c in string.chars() {
            output[i] = c2n(c);
            i += 1;
        }
        output
    }

    pub fn inv_mapping_arr26(arr: [u8;26]) -> [u8;26] {
        let mut inv: [u8;26] = [0;26];
        for i in 0..26 {
            inv[arr[i] as usize] = i as u8
        }
    
        inv
    }
}
