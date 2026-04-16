//! `BTreeMap` の反復順が決定的であることを示す小さな例です。

use std::collections::BTreeMap;

/// `BTreeMap` にいくつかキーを入れ、既存要素を 1 つ更新したあと、
/// ソート済みのキー/値ペアを表示します。
fn main() {
    let mut map: BTreeMap<&str, u32> = BTreeMap::new();
    map.insert("banana", 6);
    map.insert("apple", 5);
    map.insert("mango", 5);
    map.insert("bannar", 6);
    // `entry` を使うと、キーがすでに存在する場合の二重探索を避けられます。
    map.entry("banana").and_modify(|v| *v += 1);
    for (k, v) in map {
        println!("key = {}, value = {}", k, v);
    }
}
