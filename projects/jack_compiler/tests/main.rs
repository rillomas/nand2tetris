use jack_compiler::{
    generate_ioset, get_origin_name,
    parser::{self},
    tokenizer,
};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

const TEST_DIR: &str = "tests";
const DATA_DIR: &str = "data";

fn test_tokenizer(root: &PathBuf, dir: &str) -> Result<(), std::io::Error> {
    let target = root.join(TEST_DIR).join(DATA_DIR).join(dir);
    // println!("{:?}", target);
    // Convert jack to token xml for each directory
    let io_list = generate_ioset(&target)?;
    for mut io in io_list {
        let origin = get_origin_name(&io.input_file).unwrap();
        let mut golden_file_path = io.input_file.clone();
        let golden_name = format!("{}T.xml", origin);
        golden_file_path.set_file_name(&golden_name);
        let tokens = tokenizer::generate_token_list(&mut io.input);

        // Read Golden XML results and compare with results
        let golden_xml = std::fs::read_to_string(golden_file_path).unwrap();
        let xml = tokens.serialize().unwrap();
        println!("{} vs {}", &golden_name, io.input_file.display());
        assert_eq!(golden_xml, xml);
    }
    Ok(())
}

fn test_parser(root: &PathBuf, dir: &str) {
    let target = root.join(TEST_DIR).join(DATA_DIR).join(dir);
    // println!("{:?}", target);
    // Convert jack to parsed xml for each directory
    let io_list = generate_ioset(&target).unwrap();
    for mut io in io_list {
        let origin = get_origin_name(&io.input_file).unwrap();
        let mut golden_file_path = io.input_file.clone();
        let golden_name = format!("{}.xml", origin);
        golden_file_path.set_file_name(&golden_name);
        let mut ctx = parser::Context::new();
        let class = parser::parse_file(&mut ctx, &mut io.input)
            .expect(format!("Parse failed at {}", io.input_file.display()).as_str());

        // Read Golden XML results and compare with results
        let golden_xml = std::fs::read_to_string(golden_file_path).unwrap();
        let mut xml = String::from("");
        class.serialize(&mut xml, 0).unwrap();
        // println!("{}", golden_xml);
        // println!("{}", xml);
        assert_eq!(golden_xml, xml);
        println!("OK: {} vs {}", &golden_name, io.input_file.display());
    }
}

fn test_compiler(root: &PathBuf, dir: &str) {
    let target = root.join(TEST_DIR).join(DATA_DIR).join(dir);
    // Convert jack to parsed xml for each directory
    let io_list = generate_ioset(&target).unwrap();
    for mut io in io_list {
        let origin = get_origin_name(&io.input_file).unwrap();
        let mut output_file_path = io.input_file.clone();
        let output_name = format!("{}.vm", origin);
        output_file_path.set_file_name(&output_name);
        let mut ctx = parser::Context::new();
        let class = parser::parse_file(&mut ctx, &mut io.input)
            .expect(format!("Parse failed at {}", io.input_file.display()).as_str());

        // Write to VM file
        let file = File::create(output_file_path).unwrap();
        let mut writer = BufWriter::new(file);
        let res = class.compile_to(&ctx, &mut writer);
        assert!(res.is_ok());
    }
}

#[test]
fn test_tokenized_array_test_xml() -> Result<(), std::io::Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_tokenizer(&root, "ArrayTest")
}

#[test]
fn test_tokenized_expression_less_square_xml() -> Result<(), std::io::Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_tokenizer(&root, "ExpressionLessSquare")
}

#[test]
fn test_tokenized_square_xml() -> Result<(), std::io::Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_tokenizer(&root, "Square")
}

#[test]
fn test_parser_expression_less_square_xml() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_parser(&root, "ExpressionLessSquare");
}

#[test]
fn test_parser_array_test_xml() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_parser(&root, "ArrayTest");
}

#[test]
fn test_parser_square_xml() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_parser(&root, "Square");
}

#[test]
fn test_compiler_seven() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_compiler(&root, "Seven");
}
