use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::{fs::File, io::{Read, Write}, process::exit};
use zstd::{bulk::compress, stream::decode_all};
use indicatif::{ProgressBar, ProgressStyle};
use clap::{Parser, Subcommand};
use rand::{rngs::ThreadRng, Rng};

#[derive(Parser)]
#[command(author = "gpchn")]
#[command(version = "0.1.1")]
#[command(about = "使用异或算法加密文件", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    actions: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    /// 加密（Encrypt）
    Enc(EncArgs),
    /// 解密（Decrypt）
    Dec(DecArgs),
}

#[derive(Parser)]
struct EncArgs {
    /// 输入（文本/路径）
    input: Option<String>,

    /// 使用文本作为输入
    #[clap(short='t', long="text", default_value_t = false)]
    text: bool,

    /// 输出路径
    #[clap(short='o', long="output", default_value = "out")]
    output: Option<String>,

    /// 直接输出，不保存
    #[clap(short='p', long="print", default_value_t = false)]
    print: bool,
}

#[derive(Parser)]
struct DecArgs {
    /// 输入（文本/路径）
    input: Option<String>,

    /// 使用文本作为输入（用空格分割密文和密钥）
    #[clap(short='t', long="text", default_value_t = false)]
    text: bool,

    /// 输出路径
    #[clap(short='o', long="output", default_value = "out")]
    output: Option<String>,

    /// 直接输出，不保存
    #[clap(short='p', long="print", default_value_t = false)]
    print: bool,
}

fn main() {
    let args = Cli::parse();

    match args.actions {
        Some(Action::Enc(enc_args)) => handle_encrypt(enc_args),
        Some(Action::Dec(dec_args)) => handle_decrypt(dec_args),
        None => eprintln!("No action specified"),
    }
}

fn handle_encrypt(enc_args: EncArgs) {
    let output: String = enc_args.output.unwrap();
    let buffer: Vec<u8> = if enc_args.text {
        let text: String = enc_args.input.expect("待加密文本不合法");
        let buffer: Vec<u8> = BASE64.encode(text).as_bytes().to_vec();
        println!("正在压缩数据...");
        compress(&buffer, 0).expect("压缩失败")
    } else {
        let path: String = enc_args.input.expect("待加密文件路径不合法");
        println!("正在读取文件：{path}...");
        let buffer: Vec<u8> = read_file(&path);
        println!("正在压缩数据...");
        compress(&buffer, 0).expect("压缩失败")
    };

    let (result, key) = encrypt(&buffer);

    if enc_args.print {
        let result_b64: String = BASE64.encode(result);
        let key_b64: String = BASE64.encode(key);
        println!("\n加密完成！\n密文：{result_b64:?}\n密钥：{key_b64:?}");
    } else {
        println!("\n加密完成！正在保存文件...");
        save_to_file(&output, &result);
        save_to_file(&(output.clone() + ".key"), &key);
        println!("文件已保存在 {output}");
    }
}

fn handle_decrypt(dec_args: DecArgs) {
    let output: String = dec_args.output.unwrap();
    let (buffer, key) = if dec_args.text {
        let input: String = dec_args.input.unwrap();
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 2 {
            eprintln!("输入格式不正确，应该包含密文和密钥");
            exit(1);
        }
        let buffer: Vec<u8> = BASE64.decode(parts[0]).expect("解码密文失败");
        let key: Vec<u8> = BASE64.decode(parts[1]).expect("解码密钥失败");
        println!("正在解密数据...");
        (buffer, key)
    } else {
        let path: String = dec_args.input.expect("输出路径不合法");
        println!("正在读取文件：{path}...");
        let buffer: Vec<u8> = read_file(&path);
        println!("正在读取密钥：{path}.key...");
        let key: Vec<u8> = read_file(&(path + ".key"));
        println!("正在解密数据...");
        (buffer, key)
    };

    let result: Vec<u8> = decrypt(&buffer, &key);

    if dec_args.print {
        let result_str: String = String::from_utf8(result).expect("解码失败");
        println!("解压完成！\n原文：{result_str:?}");
    } else {
        println!("正在保存文件...");
        save_to_file(&output, &result);
        println!("文件已保存在 {output}");
    }
}

fn encrypt(buffer: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut key: Vec<u8> = Vec::new();
    let mut result: Vec<u8> = Vec::new();
    let pbar: ProgressBar = create_progress_bar(buffer.len() as u64);

    for &b in buffer {
        let rb: u8 = random_byte();
        key.push(rb);
        result.push(b ^ rb);
        pbar.inc(1);
    }

    pbar.finish();
    (result, key)
}

fn decrypt(buffer: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let pbar: ProgressBar = create_progress_bar(buffer.len() as u64);

    for i in 0..buffer.len() {
        result.push(buffer[i] ^ key[i]);
        pbar.inc(1);
    }

    pbar.finish();
    println!("解密完成！正在解压...");
    let decompressed: Vec<u8> = decode_all(&result[..]).expect("解压失败");
    BASE64.decode(&decompressed[..]).expect("解码失败")
}

fn read_file(path: &str) -> Vec<u8> {
    let mut file: File = File::open(path).unwrap_or_else(|e| {
        eprintln!("无法读取文件: {path}\n错误信息：{e}");
        exit(1);
    });
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).expect("读取文件失败");
    buffer
}

fn save_to_file(path: &str, data: &[u8]) {
    let mut file: File = File::create(path).expect("无法创建文件");
    file.write_all(data).expect("无法写入文件");
}

fn random_byte() -> u8 {
    let mut rng: ThreadRng = ThreadRng::default();
    rng.random()
}

fn create_progress_bar(length: u64) -> ProgressBar {
    let pbar = ProgressBar::new(length);
    pbar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{wide_bar:.cyan/blue}] {percent}% ({eta})")
        .expect("进度条样式设置失败")
        .progress_chars("#>-"));
    pbar
}
