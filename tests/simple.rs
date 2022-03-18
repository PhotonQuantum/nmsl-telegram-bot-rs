use std::io::Cursor;

use nmsl_core::SunBible;

static BIBLE: &[u8] = include_bytes!("../bible.json");

#[test]
fn must_generate_emoji() {
    let bible = SunBible::new_from_reader(Cursor::new(BIBLE)).expect("load bible");
    assert_eq!(
        bible.convert("我质疑你妈死了，我是抽象大师，这就是二次元吗，爱了爱了。"),
        "👴📄1️⃣你🐴💀🌶️，👴🔟抽🐘带师，这9️⃣🔟二刺螈🐴，ilil。"
    );
}
