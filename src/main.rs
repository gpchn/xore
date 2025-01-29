use std::{fs::File, io::{Read, Write}, process::exit};
use rand::{rngs::ThreadRng, Rng};
use clap::{ArgGroup, Parser};


#[derive(Parser)]
#[command(author = "gpchn")]
#[command(version = "0.1.0")]
#[command(about = "Use xor to encrypt file", long_about = None)]
#[clap(group(ArgGroup::new("actions")
    .args(&["encrypt", "decrypt"])
    .required(true)))]
struct Cli {
    // 添加互斥组
    /// Path to the file to encrypt
    #[clap(short='e', long="encrypt")]
    encrypt: Option<String>,

    /// Path to the file to decrypt
    #[clap(short='d', long="decrypt")]
    decrypt: Option<String>,

    /// Output file
    #[clap(short='o', long="output", default_value = "out")]
    output: Option<String>,
}


fn main() {
    // 获取命令行参数
    let args = Cli::parse();

    if args.encrypt.is_some() {
        // 读取文件二进制数据
        let buffer: Vec<u8> = read_file(args.encrypt.unwrap().as_str()).unwrap();
        encrypt(&buffer, &args.output.unwrap());
    } else {
        let path = args.decrypt.unwrap();
        // 读取文件二进制数据
        let buffer: Vec<u8> = read_file(path.as_str()).unwrap();
        // 读取随机数文件二进制数据
        let buffer_rand: Vec<u8> = read_file(&(path + ".key")).unwrap();
        decrypt(&buffer, &buffer_rand, &args.output.unwrap());
    }
}


fn encrypt(buffer: &Vec<u8>, output: &str) {
    // 初始化
    let mut rand_buf: Vec<u8> = Vec::new();
    let mut result: Vec<u8> = Vec::new();

    // 开始加密
    buffer.iter().for_each(|&b| {
        let rb = random_byte();
        rand_buf.push(rb);
        result.push(b ^ rb);
    });

    // 保存加密后的文件
    let mut file: File = File::create(output).unwrap();
    file.write_all(&result).unwrap();
    // 保存随机数
    let mut file: File = File::create(output.to_string() + ".key").unwrap();
    file.write_all(&rand_buf).unwrap();
}


fn decrypt(buffer: &Vec<u8>, buffer_rand: &Vec<u8>, output: &str) {
    // 开始解密
    let mut result: Vec<u8> = Vec::new();
    for i in 0..buffer.len() {
        result.push(buffer[i] ^ buffer_rand[i]);
    }

    // 保存解密后的文件
    let mut file: File = File::create(output).unwrap();
    file.write_all(&result).unwrap();
}


fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file: File = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            exit(1);
        }
    };
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}


fn random_byte() -> u8 {
    let mut rng: ThreadRng = ThreadRng::default();
    return rng.random();
}