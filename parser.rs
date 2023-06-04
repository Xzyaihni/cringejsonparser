use std::{
    ops::Index,
    str::Chars,
    iter::Peekable
};


fn verify_char(mut text: impl Iterator<Item=char>, expected: char)
{
    let mut this = text.next();
    if this.map(|c| c.is_whitespace()).unwrap_or(false)
    {
        this = text.next();
    }

    if this != Some(expected)
    {
        panic!(
            "tried to parse the wrong object (expected '{expected}' got {})",
            this.map(|c| format!("'{c}'")).unwrap_or_else(|| "none".to_owned())
        );
    }
}

fn parse_text(mut text: &mut TextIter) -> String
{
    verify_char(&mut text, '"');

    text.take_while(move |c| *c != '"').collect()
}

#[derive(Debug)]
pub enum ObjectValue
{
    Text(String),
    Number(u32),
    Bool(bool),
    List(Box<[ObjectValue]>),
    Object(Box<Object>)
}

impl ObjectValue
{
    #[allow(dead_code)]
    pub fn get_text(&self) -> Option<&str>
    {
        match self
        {
            ObjectValue::Text(x) => Some(x),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn get_number(&self) -> Option<u32>
    {
        match self
        {
            ObjectValue::Number(x) => Some(*x),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn get_bool(&self) -> Option<bool>
    {
        match self
        {
            ObjectValue::Bool(x) => Some(*x),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn get_list(&self) -> Option<&[ObjectValue]>
    {
        match self
        {
            ObjectValue::List(x) => Some(x),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn get_object(&self) -> Option<&Object>
    {
        match self
        {
            ObjectValue::Object(x) => Some(x),
            _ => None
        }
    }

    pub fn parse(text: &mut TextIter) -> Self
    {
        let beginning = text.peek().expect("text must not be empty");

        match beginning.to_ascii_lowercase()
        {
            '"' => Self::parse_text(text),
            '[' => Self::parse_list(text),
            '{' => Self::parse_object(text),
            'f' | 't' => Self::parse_bool(text),
            n if n.is_digit(10) => Self::parse_number(text),
            _ => panic!("unexpected token: '{}'", beginning)
        }
    }

    fn parse_text(text: &mut TextIter) -> Self
    {
        Self::Text(parse_text(text))
    }

    fn parse_number(text: &mut TextIter) -> Self
    {
        let mut number = String::new();

        while let Some(c) = text.peek()
        {
            if !c.is_digit(10)
            {
                break;
            }

            number.push(text.next().expect("checked that its not none with peek"));
        }

        Self::Number(number.parse().expect("must be a number"))
    }

    fn parse_bool(text: &mut TextIter) -> Self
    {
        let beginning = text.next().expect("bool parse request must be valid")
            .to_ascii_lowercase();

        let keyword_length = match beginning
        {
            't' => 3,
            'f' => 4,
            x => panic!("unexpected token: '{}'", x)
        };

        let mut value = beginning.to_string();
        value.extend(text.take(keyword_length));

        let value = match value.to_lowercase().as_ref()
        {
            "true" => true,
            "false" => false,
            x => panic!("invalid boolean value: {}", x)
        };

        Self::Bool(value)
    }

    fn parse_list(mut text: &mut TextIter) -> Self
    {
        verify_char(&mut text, '[');

        let mut values = Vec::new();

        while let Some(c) = text.peek()
        {
            if *c == ']'
            {
                break;
            }

            if !values.is_empty()
            {
                verify_char(&mut text, ',');
            }

            let value = ObjectValue::parse(&mut text);

            values.push(value);
        }

        verify_char(text, ']');

        Self::List(values.into_boxed_slice())
    }

    fn parse_object(text: &mut TextIter) -> Self
    {
        Self::Object(Box::new(Object::parse(text)))
    }
}

impl Index<usize> for ObjectValue
{
    type Output = ObjectValue;

    fn index(&self, id: usize) -> &Self::Output
    {
        match self
        {
            ObjectValue::List(list) => &list[id],
            x => panic!("cant index into a value of type: {:?}", x)
        }
    }
}

#[derive(Debug)]
pub struct ObjectField
{
    key: String,
    value: ObjectValue
}

impl ObjectField
{
    pub fn parse(mut text: &mut TextIter) -> Self
    {
        let key = parse_text(text);

        verify_char(&mut text, ':');

        let value = ObjectValue::parse(text);

        Self{key, value}
    }

    #[allow(dead_code)]
    pub fn key(&self) -> &str
    {
        &self.key
    }

    #[allow(dead_code)]
    pub fn value(&self) -> &ObjectValue
    {
        &self.value
    }
}

#[derive(Debug)]
pub struct Object
{
    fields: Box<[ObjectField]>
}

impl Object
{
    pub fn parse(mut text: &mut TextIter) -> Self
    {
        verify_char(&mut text, '{');

        let mut fields = Vec::new();

        while let Some(c) = text.peek()
        {
            if *c == '}'
            {
                break;
            }

            if !fields.is_empty()
            {
                verify_char(&mut text, ',');
            }

            let field = ObjectField::parse(&mut text);

            fields.push(field);
        }

        verify_char(text, '}');

        Self{fields: fields.into_boxed_slice()}
    }

    #[allow(dead_code)]
    pub fn fields(&self) -> &[ObjectField]
    {
        &self.fields
    }
}

impl Index<&str> for Object
{
    type Output = ObjectValue;

    fn index(&self, id: &str) -> &Self::Output
    {
        self.fields.iter().find_map(|field|
        {
            if AsRef::<str>::as_ref(&field.key) == id
            {
                Some(&field.value)
            } else
            {
                None
            }
        }).unwrap_or_else(|| panic!("field with key \"{}\" not found", id))
    }
}

type TextIterInner<'a> = Chars<'a>;
type TextIter<'a> = Peekable<TextIterInner<'a>>;

pub struct Parser<'a>
{
    text: TextIter<'a>
}

impl<'a> Parser<'a>
{
    pub fn new(text: TextIterInner<'a>) -> Self
    {
        Self{text: text.peekable()}
    }

    pub fn parse(mut self) -> ObjectValue
    {
        ObjectValue::parse(&mut self.text)
    }
}