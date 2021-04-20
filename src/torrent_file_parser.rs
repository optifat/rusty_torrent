use std::fs::read;
use std::io;
use std::collections::HashMap;
use std::borrow::Cow;

// https://habr.com/ru/post/119753/
// https://github.com/jcul/bencode
// https://en.wikipedia.org/wiki/Bencode

#[derive(PartialEq, Eq)]
pub enum Content{
    Str(String),
    List(Vec<Content>),
    Int(i64),
    Dict(HashMap<String, Content>),
}

pub fn parse_torrent_file(filename: String) -> Result<Vec<Content>, io::Error>{
    let mut torrent_contents = Vec::<Content>::new();
    let binary_contents = read(filename)?;
    let string_contents = String::from_utf8_lossy(&binary_contents);
    let string_contents_length = string_contents.chars().count();

    Ok(torrent_contents)
}

fn parse_int(contents: &Cow<str>, current_index: &mut usize) -> i64{
    let mut str_num = String::new();
    let mut symbol = contents.chars().nth(*current_index).unwrap();
    while symbol != 'e' {
        str_num.push(symbol);
        *current_index += 1;
        symbol = contents.chars().nth(*current_index).unwrap();
    }
    *current_index += 1;
    str_num.parse::<i64>().unwrap()
}

fn parse_string(contents: &Cow<str>, current_index: &mut usize) -> String{
    let mut len_str = String::new();
    let mut symbol = contents.chars().nth(*current_index).unwrap();
    while symbol != ':' {
        len_str.push(symbol);
        *current_index += 1;
        symbol = contents.chars().nth(*current_index).unwrap();
    }
    *current_index += 1;
    let mut string = String::new();
    let len_str = len_str.parse::<usize>().unwrap();
    for i in 0..len_str{
        string.push(contents.chars().nth(*current_index).unwrap());
        *current_index += 1;
    }

    string
}

fn parse_list(contents: &Cow<str>, current_index: &mut usize) -> Vec<Content>{
    let mut list = Vec::<Content>::new();
    let mut symbol = contents.chars().nth(*current_index).unwrap();
    while symbol != 'e' {
        if symbol == 'i'{
            *current_index += 1;
            list.push(Content::Int(parse_int(contents, current_index)));
        }
        else if symbol.is_digit(10){
            list.push(Content::Str(parse_string(contents, current_index)));
        }
        else if symbol == 'l'{
            *current_index += 1;
            list.push(Content::List(parse_list(contents, current_index)));
        }
        else if symbol == 'd'{
            *current_index += 1;
            list.push(Content::Dict(parse_dict(contents, current_index)));
        }
        else{
            panic!("Unknown type {}", symbol);
        }
        symbol = contents.chars().nth(*current_index).unwrap();
    }

    list
}

fn parse_dict(contents: &Cow<str>, current_index: &mut usize) -> HashMap<String, Content>{
    let mut dict_content = HashMap::<String, Content>::new();
    let mut key = String::from("");
    let mut reading_key = true;
    let mut symbol = contents.chars().nth(*current_index).unwrap();

    while symbol != 'e' {
        if symbol == 'i'{
            *current_index += 1;
            if reading_key{
                panic!("Dictionary keys must be byte strings");
            }
            else{
                dict_content.insert(key.clone(), Content::Int(parse_int(contents, current_index)));
                reading_key = true;
            }
        }
        else if symbol.is_digit(10){
            if reading_key{
                key = parse_string(contents, current_index);
                reading_key = false;
                if dict_content.get(&key).is_some(){
                    panic!("Dictionary has a duplicate key");
                }
            }
            else{
                dict_content.insert(key.clone(), Content::Str(parse_string(contents, current_index)));
                reading_key = true;
            }
        }
        else if symbol == 'l'{
            *current_index += 1;
            if reading_key{
                panic!();
            }
            else{
                dict_content.insert(key.clone(), Content::List(parse_list(contents, current_index)));
                reading_key = true;
            };
        }
        else if symbol == 'd'{
            *current_index += 1;
            if reading_key{
                panic!("Dictionary keys must be byte strings");
            }
            else{
                dict_content.insert(key.clone(), Content::Dict(parse_dict(contents, current_index)));
                reading_key = true;
            };
        }
        else{
            panic!("Unknown type {}", symbol);
        }
        symbol = contents.chars().nth(*current_index).unwrap();
    }

    dict_content
}

#[cfg(test)]

mod tests{
    #[cfg(test)]

    use super::*;

    // our functions are getting lines without starting letters, so the examples are like "42e" instead of "i42e"
    #[test]
    fn parsing_positive_int(){
        unsafe{
            assert_eq!(crate::torrent_file_parser::parse_int(&String::from_utf8_lossy(String::from("42e").as_mut_vec()), &mut 0), 42);
        }
    }

    #[test]
    fn parsing_zero_int(){
        unsafe{
            assert_eq!(crate::torrent_file_parser::parse_int(&String::from_utf8_lossy(String::from("0e").as_mut_vec()), &mut 0), 0);
        }
    }

    #[test]
    fn parsing_negative_int(){
        unsafe{
            assert_eq!(crate::torrent_file_parser::parse_int(&String::from_utf8_lossy(String::from("-75637e").as_mut_vec()), &mut 0), -75637);
        }
    }

    #[test]
    #[should_panic]
    fn parsing_negative_zero(){
        unsafe{
            crate::torrent_file_parser::parse_int(&String::from_utf8_lossy(String::from("-0e").as_mut_vec()), &mut 0);
        }
    }

    #[test]
    #[should_panic]
    fn parsing_starting_zero_in_positive(){
        unsafe{
            crate::torrent_file_parser::parse_int(&String::from_utf8_lossy(String::from("07562e").as_mut_vec()), &mut 0);
        }
    }

    #[test]
    #[should_panic]
    fn parsing_starting_zero_in_negative(){
        unsafe{
            crate::torrent_file_parser::parse_int(&String::from_utf8_lossy(String::from("-54125e").as_mut_vec()), &mut 0);
        }
    }

    #[test]
    fn parsing_string_1(){
        unsafe{
            assert_eq!(crate::torrent_file_parser::parse_string(&String::from_utf8_lossy(String::from("4:spam").as_mut_vec()), &mut 0), "spam");
        }
    }

    #[test]
    fn parsing_string_2(){
        unsafe{
            assert_eq!(crate::torrent_file_parser::parse_string(&String::from_utf8_lossy(String::from("13:parrot sketch").as_mut_vec()), &mut 0), "parrot sketch");
        }
    }
}
