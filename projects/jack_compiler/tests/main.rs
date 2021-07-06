use jack_compiler::{
    generate_ioset, get_origin_name,
    parser::{self},
    tokenizer,
};
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
        let mut ctx = parser::ClassParseInfo::new();
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

fn test_compiler(root: &PathBuf, dir: &str, print_xml: bool, print_vm: bool, compare_vm: bool) {
    let target = root.join(TEST_DIR).join(DATA_DIR).join(dir);
    // Convert jack to parsed xml for each directory
    let io_list = generate_ioset(&target).unwrap();
    let mut dir_info = jack_compiler::parser::DirectoryParseInfo::new();
    let mut class_list = Vec::new();
    for mut io in io_list {
        println!("Parsing {}", io.input_file.display());
        // let mut output_file_path = io.input_file.clone();
        // let output_name = format!("{}.vm", origin);
        // output_file_path.set_file_name(&output_name);
        let mut ctx = parser::ClassParseInfo::new();
        let class = parser::parse_file(&mut ctx, &mut io.input)
            .expect(format!("Parse failed at {}", io.input_file.display()).as_str());
        dir_info.info_per_class.insert(class.name().to_owned(), ctx);

        if print_xml {
            let mut xml = String::from("");
            class.serialize(&mut xml, 0).unwrap();
            println!("{}", xml);
        }
        class_list.push((class, io.input_file));
    }
    for (c, input_file) in class_list {
        // Compile to vm text
        let vm = c
            .compile(&dir_info)
            .expect(format!("Compile failed at {}", input_file.display()).as_str());
        if print_vm {
            println!("{}", vm);
        }

        if compare_vm {
            // Compare with golden
            let origin = get_origin_name(&input_file).unwrap();
            let mut golden_file_path = input_file.clone();
            let golden_name = format!("{}Gold.vm", origin);
            golden_file_path.set_file_name(&golden_name);
            let golden_vm = std::fs::read_to_string(golden_file_path).unwrap();
            assert_eq!(golden_vm, vm);
            println!("OK: {} vs {}", &golden_name, input_file.display());
        }
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
    test_compiler(&root, "Seven", false, false, true);
}

#[test]
fn test_compiler_convert_to_bin() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_compiler(&root, "ConvertToBin", false, false, false);
}

#[test]
fn test_compiler_square() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_compiler(&root, "Square2", false, false, false);
}
