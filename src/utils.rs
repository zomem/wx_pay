pub fn get_slice_arr<'a>(str_data: String) -> Box<[u8]> {
    let arr = match str_data.len() {
        1 => {
            let mut a = [0u8; 1];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        2 => {
            let mut a = [0u8; 2];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        3 => {
            let mut a = [0u8; 3];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        4 => {
            let mut a = [0u8; 4];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        5 => {
            let mut a = [0u8; 5];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        6 => {
            let mut a = [0u8; 6];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        7 => {
            let mut a = [0u8; 7];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        8 => {
            let mut a = [0u8; 8];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        9 => {
            let mut a = [0u8; 9];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        10 => {
            let mut a = [0u8; 10];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        11 => {
            let mut a = [0u8; 11];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        12 => {
            let mut a = [0u8; 12];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        13 => {
            let mut a = [0u8; 13];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        14 => {
            let mut a = [0u8; 14];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        15 => {
            let mut a = [0u8; 15];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        16 => {
            let mut a = [0u8; 16];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
        _ => {
            let mut a = [0u8; 0];
            hex::decode_to_slice(hex::encode(str_data), &mut a as &mut [u8]).unwrap();
            a.to_vec()
        }
    };
    arr.into_boxed_slice()
}
