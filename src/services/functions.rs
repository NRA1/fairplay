pub fn median(nums: &mut Vec<u8>) -> u8 {
    nums.sort();
    let mid = nums.len() / 2;

    if nums.len() % 2 == 0 {
        ((nums[mid - 1] as u16 + nums[mid] as u16) / 2u16) as u8
    } else {
        nums[mid]
    }
}

pub fn pitagora(x: i16, y: i16) -> i16 {
    let sum = (x as i64).pow(2) + (y as i64).pow(2);
    (sum as f32).sqrt().round() as i16
}