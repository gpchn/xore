use std::{fs::File, io::{Read, Write}, process::exit};
use zstd::{bulk::compress, stream::decode_all};
use indicatif::{ProgressBar, ProgressStyle};
use rand::{rngs::ThreadRng, Rng};
use clap::{ArgGroup, Parser};


#[derive(Parser)]
#[command(author = "gpchn")]
#[command(version = "0.1.0")]
#[command(about = "异或随机数据来加密文件", long_about = None)]
#[clap(group(ArgGroup::new("actions")
    .args(&["encrypt", "decrypt"])
    .required(true)))]
struct Cli {
    // 添加互斥组
    /// 待加密文件的路径
    #[clap(short='e', long="encrypt")]
    encrypt: Option<String>,

    /// 待解密文件的路径（密钥应位于同一目录下，且比原文件多一个 .key 后缀）
    #[clap(short='d', long="decrypt")]
    decrypt: Option<String>,

    /// 输出路径（默认为 out）
    #[clap(short='o', long="output", default_value = "out")]
    output: Option<String>,
}


fn main() {
    // 获取命令行参数
    let args = Cli::parse();

    if args.encrypt.is_some() {
        let path = args.encrypt.expect("待加密文件路径不合法");
        // 读取文件二进制数据
        println!("正在读取文件：{path}...");
        let mut buffer: Vec<u8> = read_file(path.as_str());
        println!("正在压缩数据...");
        buffer = compress(&buffer, 0).expect("压缩失败");
        println!("正在加密数据...");
        encrypt(&buffer, &args.output.expect("输出路径不合法"));
    } else {
        let path = args.decrypt.expect("待解密文件路径不合法");
        // 读取文件二进制数据
        println!("正在读取文件：{path}...");
        let buffer: Vec<u8> = read_file(path.as_str());
        // 读取密钥文件二进制数据
        println!("正在读取密钥：{path}.key...");
        let buffer_rand: Vec<u8> = read_file(&(path + ".key"));
        println!("正在解密数据...");
        decrypt(&buffer, &buffer_rand, &args.output.expect("输出路径不合法"));
    }
}


fn encrypt(buffer: &Vec<u8>, output: &str) {
    // 初始化
    let mut rand_buf: Vec<u8> = Vec::new();
    let mut result: Vec<u8> = Vec::new();
    let pbar: ProgressBar = ProgressBar::new(buffer.len() as u64);
    pbar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{wide_bar:.cyan/blue}] {percent}% ({eta})")
        .expect("进度条样式设置失败")
        .progress_chars("#>-")
    );

    // 开始加密
    buffer.iter().for_each(|&b| {
        let rb = random_byte();
        rand_buf.push(rb);
        result.push(b ^ rb);
        pbar.inc(1);
    });

    // ? 用 finish_with_message 会出问题，并没有显示 msg
    pbar.finish();
    println!("\n加密完成！正在保存文件...");
    // 保存加密后的文件
    let mut file: File = File::create(output).expect("无法创建文件");
    file.write_all(&result).expect("无法写入文件");
    // 保存密钥
    let mut file: File = File::create(output.to_string() + ".key").expect("无法创建文件");
    file.write_all(&rand_buf).expect("无法写入文件");
    println!("文件已保存在 {output}");
}


fn decrypt(buffer: &Vec<u8>, buffer_rand: &Vec<u8>, output: &str) {
    // 初始化
    let mut result: Vec<u8> = Vec::new();
    let pbar: ProgressBar = ProgressBar::new(buffer.len() as u64);
    pbar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{wide_bar:.cyan/blue}] {percent}% ({eta})")
        .expect("进度条样式设置失败")
        .progress_chars("#>-")
    );

    // 开始解密
    for i in 0..buffer.len() {
        result.push(buffer[i] ^ buffer_rand[i]);
        pbar.inc(1);
    }

    // ? 用 finish_with_message 会出问题，并没有显示 msg
    pbar.finish();
    let decompressed: Vec<u8> = decode_all(&result[..]).expect("解压失败");
    // ? 不能用 zstd::bulk::decompress，该函数要求一个固定大小的缓冲区，超出就会报错
    //let decompressed = decompress(&result, buffer.len() * 10).expect("解压失败");

    println!("\n解压完成！正在保存文件...");
    // 保存解密后的文件
    let mut file: File = File::create(output).expect("无法创建文件");
    file.write_all(&decompressed).expect("无法写入文件");
    println!("文件已保存在 {output}");
}


fn read_file(path: &str) -> Vec<u8> {
    let mut file: File = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法读取文件: {path}\n错误信息：{e}");
            exit(1);
        }
    };
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("读取文件失败");
    return buffer;
}


fn random_byte() -> u8 {
    let mut rng: ThreadRng = ThreadRng::default();
    return rng.random();
}