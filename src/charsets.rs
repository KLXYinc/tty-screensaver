#[derive(Clone)]
pub struct CharSet {
    pub name: &'static str,
    pub chars: Vec<char>,
}
impl CharSet {
    pub fn sample(&self, val: f64) -> char {
        if self.chars.is_empty() {
            return '█';
        }
        let idx = (val * self.chars.len() as f64) as usize;
        self.chars[idx.clamp(0, self.chars.len() - 1)]
    }
}
pub fn get_all_charsets_utf8() -> Vec<CharSet> {
    vec![
        CharSet {
            name: "Classic",
            chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%&*^<>~"
                .chars()
                .collect(),
        },
        CharSet {
            name: "Symbols",
            chars: " .,-~:;=!*#$@".chars().collect(),
        },
        CharSet {
            name: "Binary",
            chars: "01".chars().collect(),
        },
        CharSet {
            name: "Hexadecimal",
            chars: "0123456789ABCDEF".chars().collect(),
        },
        CharSet {
            name: "Blocks",
            chars: " ░▒▓█".chars().collect(),
        },
        CharSet {
            name: "Katakana",
            chars: "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ"
                .chars()
                .collect(),
        },
        CharSet {
            name: "Braille",
            chars: " ⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟⠠⠡⠢⠣⠤⠥⠦⠧⠨⠩⠪⠫⠬⠭⠮⠯⠰⠱⠲⠳⠴⠵⠶⠷⠸⠹⠺⠻⠼⠽⠾⠿"
                .chars()
                .collect(),
        },
        CharSet {
            name: "Runes",
            chars:
                " ᚠᚡᚢᚣᚤᚥᚦᚧᚨᚩᚪᚫᚬᚭᚮᚯᚰᚱᚲᚳᚴᚵᚶᚷᚸᚹᚺᚻᚼᚽᚾᚿᛀᛁᛂᛃᛄᛅᛆᛇᛈᛉᛊᛋᛌᛍᛎᛏᛐᛑᛒᛓᛔᛕᛖᛗᛘᛙᛚᛛᛜᛝᛞᛟᛠᛡᛢᛣᛤᛥᛦᛧᛨᛩᛪ᛫᛬᛭ᛮᛯ"
                    .chars()
                    .collect(),
        },
        CharSet {
            name: "Math",
            chars: " ∑∫∯∰∮∲∳∀∁∂∃∄∅∆∇∈∉∊∋∌∍∎∏∐".chars().collect(),
        },
        CharSet {
            name: "Greek",
            chars: " αβγδεζηθικλμνξοπρστυφχψωΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩ"
                .chars()
                .collect(),
        },
        CharSet {
            name: "Cyrillic",
            chars: " абвгдежзийклмнопрстуфхцчшщъыьэюяАБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ"
                .chars()
                .collect(),
        },
        CharSet {
            name: "Cards",
            chars: " ♠♡♢♣♤♥♦♧".chars().collect(),
        },
        CharSet {
            name: "Arrows",
            chars: " ←↑→↓↔↕↖↗↘↙↚↛↜↝↞↟↠↡↢↣↤↥↦↧↨".chars().collect(),
        },
    ]
}
