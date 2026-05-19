#[derive(Clone)]
pub struct CharSet {
    pub name: &'static str,
    pub chars: Vec<char>,
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
            name: "Emoji",
            chars: " 💀👻👽👾🤖🎃😈👿👹👺💥🔥✨🌟💫".chars().collect(),
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
