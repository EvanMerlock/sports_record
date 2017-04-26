#[derive(Serialize, Deserialize, Debug)]
pub struct InputInfo {
    pub height: i32,
    pub width: i32,

    pub timebase_num: i32,
    pub timebase_den: i32,

    pub framerate_num: i32,
    pub framerate_den: i32,
}

impl InputInfo {
    pub fn new(h: i32, w: i32, tb_n: i32, tb_d: i32, fr_n: i32, fr_d: i32) -> InputInfo {
        InputInfo {
            height: h,
            width: w,
            timebase_num: tb_n,
            timebase_den: tb_d,
            framerate_num: fr_n,
            framerate_den: fr_d,
        }
    }
}