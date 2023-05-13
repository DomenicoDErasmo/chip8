pub mod keyboard {
    use crossterm::event::{read, Event, KeyCode, KeyEvent};
    use crossterm::terminal::enable_raw_mode;
    use std::collections::HashSet;

    pub fn parse_keyboard() -> Option<char> {
        enable_raw_mode().unwrap();
        let valid_characters = init_hashset();

        read()
            .ok()
            .and_then(|pressed| -> Option<char> { parse_character(pressed, &valid_characters) })
    }

    fn parse_character(pressed: Event, valid_characters: &HashSet<char>) -> Option<char> {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            ..
        }) = pressed
        {
            valid_characters.contains(&ch).then_some(ch)
        } else {
            None
        }
    }

    fn init_hashset() -> HashSet<char> {
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
