pub mod keyboard {

    use std::collections::HashSet;

    pub fn parse_keyboard() {
        // TODO
    }

    fn _init_hashset() -> HashSet<char> {
        // 1234
        // qwer
        // asdf
        // zxcv

        // with p as a kill switch
        HashSet::<char>::from([
            '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v', 'p',
        ])
    }
}
