//!
//! wavファイルをリネームするプログラム
//!
use std::collections::HashMap;
use std::env;
use std::fs::{read_dir, remove_file, rename, DirEntry, File};
use std::hash::Hash;
use std::io::Read;
use std::path::Path;
use encoding_rs::SHIFT_JIS;
use encoding_rs_io::DecodeReaderBytesBuilder;

fn main() {
    let file_path = get_first_grg();
    let Some(directory) = file_path else {
        println!("引数にディレクトリが指定されていません。");
        return;
    };

    let txt_files = get_files_from_directory(&directory, ".txt");
    let Some(txt_files) = txt_files else {
        println!("指定されたディレクトリにtxtファイルが存在しません。");
        return;
    };
    let wav_files = get_files_from_directory(&directory,".wav");
    let Some(mut wav_files) = wav_files else {
        println!("指定されたディレクトリにwavファイルが存在しません。");
        return;
    };
    // ファイル名でソート
    wav_files.sort_by(|a, b| {
        let a_name_binding = a.file_name();
        let b_name_binding = b.file_name();

        let a_name = a_name_binding.to_string_lossy();
        let b_name = b_name_binding.to_string_lossy();

        // まず文字列の長さで比較し、同じ長さなら辞書順で比較
        a_name.len().cmp(&b_name.len()).then(a_name.cmp(&b_name))
    });

    // テキストファイルから新しいwavファイル名を取得
    let name_map= get_new_wav_filename_map(txt_files);

    // wavファイルをリネーム
    rename_wav_files(&directory, wav_files, name_map);

    println!("リネームが完了しました。");
}

///
/// 第１引数を取得します
///
/// @return Option<String> 第１引数
fn get_first_grg() -> Option<String> {
    Some(env::args().collect::<Vec<String>>().get(1)?.clone())
}

///
/// 指定されたディレクトリから指定された拡張子のファイルを取得します
///
/// @param path ディレクトリのパス
/// @param extension 拡張子
///
/// @return Option<Vec<DirEntry>> ファイルのリスト
fn get_files_from_directory(path: &str, extension: &str) -> Option<Vec<DirEntry>> {
    let directory = read_dir(path);
    let Ok(directory) = directory else {
        return None;
    };
    let result: Vec<DirEntry> = directory.filter_map(|entry| {
        let entry = entry.ok()?;
        let filename_bind = entry.file_name();
        let filename = filename_bind.to_str()?;
        if !filename.contains(extension) {
            return None;
        }
        Some(entry)
    }).collect();

    Some(result)
}

///
/// テキストファイルから新しいwavファイル名のベースを取得します
/// テキストファイルはSHIFT_JISでエンコードされていることを前提とします
/// テキストファイルは読み込んだ後削除します
///
/// @param txt_files テキストファイルのリスト
///
/// @return HashMap<String, String> 新しいwavファイル名のベースとなるマップ
fn get_new_wav_filename_map(txt_files: Vec<DirEntry>) -> HashMap<String, String> {
    // テキストファイルから新しいwavファイル名を取得
    let mut name_map: HashMap<String, String> = HashMap::new();
    txt_files.iter().for_each(|entry| {
        // ファイルを開く
        let file = File::open(entry.path());
        let Ok(file) = file else {
            return;
        };

        // SHIFT_JISで読み込む
        let mut content = String::new();
        let mut decoder = DecodeReaderBytesBuilder::new()
            .encoding(Some(SHIFT_JIS))
            .build(file);
        let _ = decoder.read_to_string(&mut content);

        // ファイル名を取得
        let filename = entry.file_name();
        let Some(filename) = filename.to_str() else {
            return;
        };
        // 拡張子を削除
        let item_name = filename.replace(".txt", "");

        // HashMapに格納
        name_map.insert(item_name, content);

        // テキストファイルを削除
        let _ = remove_file(entry.path());
    });

    name_map
}

///
/// wavファイルをリネームします
/// ファイル名が20文字を超える場合は切り取ります
/// ファイル名は連番を付与します
///
/// @param directory ディレクトリのパス
/// @param wav_files wavファイルのリスト
/// @param name_map 新しいwavファイル名のベースとなるマップ
///
fn rename_wav_files(directory: &str, wav_files: Vec<DirEntry>, name_map: HashMap<String, String>)  {
    // wavファイルをリネーム
    let mut count = 0;
    wav_files.iter().for_each(|entry| {
        // ファイル名を取得
        let filename = entry.file_name();
        let Some(filename) = filename.to_str() else {
            return;
        };

        println!("{}", filename);
        // 拡張子を削除
        let item_name = filename.replace(".wav", "");

        // 新しいファイル名のベースを取得
        let new_item_name_base = name_map.get(&item_name);
        let Some(new_item_name_base) = new_item_name_base else {
            println!("{}に対応するテキストファイルがありませんでした。", item_name);
            return;
        };

        // 新しいファイル名を10文字で切り取る
        let mut new_item_name = format!("{}_{}", count, new_item_name_base);
        count += 1;
        if new_item_name.chars().count() > 20 {
            new_item_name = new_item_name.chars().take(20).collect::<String>();
        }
        new_item_name.push_str(".wav");

        let after_rename_path = Path::new(&directory).join(&new_item_name);
        let _ = rename(entry.path(), after_rename_path);
    });
}
