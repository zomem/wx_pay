
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

// const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";

/// 随机字符串 生成 ，传入生成长度。
/// #### 当长度小于等于 16 时，纯随机。
/// #### 当长度大于 16 时，会包含微秒时间值和随机字符。以保证随机唯一性。
/// ```
/// let ran_1 = rand_string(12);
/// 
/// let ran_2 = rand_string(32);
/// ```
/// 
pub fn rand_string(len: u16) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let mut ran_string: String;
    
    if len <= 16 {
        let mut rng = rand::thread_rng();
        ran_string = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    } else {
        let time_len = 16;
        let ran_len = len - time_len;
        let mut rng = rand::thread_rng();
        ran_string = (0..ran_len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

        let rand_time_string = rand_time_string();
        ran_string = ran_string + rand_time_string.as_str();
    }
    ran_string
}

fn rand_time_string() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
        .to_string();
    const CHARSET: &str = "5fk12xn8Er";
    let mut r_string = String::from("");
    for c in time.chars() {
        const RADIX: u32 = 10;
        let ic = c.to_digit(RADIX).unwrap() as usize;
        let item = CHARSET.chars().nth(ic).unwrap_or('x');
        r_string.push(item);
    }
    r_string
}