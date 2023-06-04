use std::{
    fs,
    env,
    process
};

use parser::Parser;

mod parser;


fn complain(message: &str) -> !
{
    eprintln!("{message}");

    process::exit(1)
}

fn main()
{
    let filepath = env::args().nth(1)
        .unwrap_or_else(|| complain("pls provide a path as argument"));

    let data = fs::read_to_string(filepath)
        .unwrap_or_else(|err| complain(&format!("error reading file: {err:?}")));

    let parser = Parser::new(data.chars());

    let json = parser.parse();

    for object in json.get_list().unwrap()
    {
        let object = object.get_object().unwrap();

        println!(
            "{:#x} {}",
            object["vaddr"].get_number().unwrap(),
            object["name"].get_text().unwrap()
        )
        /*for field in object.fields()
        {
            println!("{field:?}");
        }

        println!();*/
    }
}
