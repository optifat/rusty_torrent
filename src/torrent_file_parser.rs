use std::fs::read;
use std::io;
use std::collections::HashMap;
use sha1::{Sha1, Digest};
use crate::bencode_content::Content;

// https://habr.com/ru/post/119753/
// https://en.wikipedia.org/wiki/Bencode

static mut INFO_START: usize = 0;
static mut INFO_END: usize = 0;

pub fn parse_torrent_file(filename: String) -> Result<(HashMap<String, Content>, Vec<u8>), io::Error>{
    let binary_contents = read(filename)?;

    if binary_contents[0] != 'd' as u8 {
        return Err(io::Error::new(io::ErrorKind::Other, "Is it possible for .torrent file to start not from 'd'?"));
    }

    let mut current_index: usize = 1;
    let torrent_contents = parse_dict(&binary_contents, &mut current_index);
    let info_hash = create_info_hash(&binary_contents);
    Ok((torrent_contents, info_hash))
}

fn create_info_hash(contents: &Vec<u8>) -> Vec<u8>{
    let mut hasher = Sha1::new();
    unsafe {
        hasher.update(&contents[INFO_START..INFO_END]);
    }
    hasher.finalize().to_vec()
}

fn parse_int(contents: &Vec<u8>, current_index: &mut usize) -> i64{
    let mut str_num = String::new();
    let mut symbol = contents[*current_index];

    while symbol != 'e' as u8 {
        str_num.push(symbol as char);
        *current_index += 1;
        symbol = contents[*current_index];
    }
    *current_index += 1;
    str_num.parse::<i64>().unwrap()
}

fn parse_bytes(contents: &Vec<u8>, current_index: &mut usize) -> Vec<u8>{
    let mut len_str = String::new();
    let mut symbol = contents[*current_index];

    while symbol != ':' as u8 {
        len_str.push(symbol as char);
        *current_index += 1;
        symbol = contents[*current_index];
    }
    let len_str = len_str.parse::<usize>().unwrap();

    *current_index += 1;
    let mut bytes = Vec::<u8>::new();
    bytes.reserve(len_str);

    for _ in 0..len_str{
        bytes.push(contents[*current_index]);
        *current_index += 1;
    }

    bytes
}

fn parse_string(contents: &Vec<u8>, current_index: &mut usize) -> String{
    String::from_utf8(parse_bytes(&contents, current_index)).unwrap()
}

fn parse_list(contents: &Vec<u8>, current_index: &mut usize) -> Vec<Content>{
    let mut list = Vec::<Content>::new();
    let mut symbol = contents[*current_index];
    while symbol != 'e' as u8{
        if symbol == 'i' as u8{
            *current_index += 1;
            list.push(Content::Int(parse_int(contents, current_index)));
        }
        else if symbol >= '0' as u8 && symbol <= '9' as u8{
            list.push(Content::Str(parse_string(contents, current_index)));
        }
        else if symbol == 'l' as u8{
            *current_index += 1;
            list.push(Content::List(parse_list(contents, current_index)));
        }
        else if symbol == 'd' as u8{
            *current_index += 1;
            list.push(Content::Dict(parse_dict(contents, current_index)));
        }
        else{
            panic!("Unknown type {}", symbol as char);
        }
        symbol = contents[*current_index];
    }
    *current_index += 1;
    list
}

fn parse_dict(contents: &Vec<u8>, current_index: &mut usize) -> HashMap<String, Content>{
    let mut dict_content = HashMap::<String, Content>::new();
    let mut key = String::from("");
    let mut reading_key = true;
    let mut symbol = contents[*current_index];
    let mut info_key_met = false;

    while symbol != 'e' as u8{

        if !info_key_met && key == "info" && !reading_key{
            info_key_met = true;
            unsafe{
                INFO_START = *current_index;
            }
        }

        if symbol == 'i' as u8{
            *current_index += 1;
            if reading_key{
                panic!("Dictionary keys must be byte strings");
            }
            else{
                dict_content.insert(key.clone(), Content::Int(parse_int(contents, current_index)));
                if info_key_met{
                    unsafe {
                        INFO_END = *current_index;
                    }
                }
                reading_key = true;
            }
        }
        else if symbol >= '0' as u8 && symbol <= '9' as u8{
            if reading_key{
                key = parse_string(contents, current_index);
                reading_key = false;
                if dict_content.get(&key).is_some(){
                    panic!("Dictionary has a duplicate key");
                }
            }
            else{
                if key != "pieces"{
                    dict_content.insert(key.clone(), Content::Str(parse_string(contents, current_index)));
                }
                else{
                    dict_content.insert(key.clone(), Content::Bytes(parse_bytes(contents, current_index)));
                }
                if info_key_met{
                    info_key_met = false;
                    unsafe {
                        INFO_END = *current_index;
                    }
                }
                reading_key = true;
            }
        }
        else if symbol == 'l' as u8{
            *current_index += 1;
            if reading_key{
                panic!();
            }
            else{
                dict_content.insert(key.clone(), Content::List(parse_list(contents, current_index)));
                if info_key_met{
                    unsafe {
                        INFO_END = *current_index;
                    }
                }
                reading_key = true;
            };
        }
        else if symbol == 'd' as u8{
            *current_index += 1;
            if reading_key{
                panic!("Dictionary keys must be byte strings");
            }
            else{
                dict_content.insert(key.clone(), Content::Dict(parse_dict(contents, current_index)));
                if info_key_met{
                    unsafe {
                        INFO_END = *current_index;
                    }
                }
                reading_key = true;
            };
        }
        else{
            panic!("Unknown type {}", symbol as char);
        }
        symbol = contents[*current_index];
    }
    *current_index += 1;

    dict_content
}

// I assume that .torrent file is OK, so I don't check some Bencode restrictions (like "i-0e" or "i-000532e" and so on)

#[cfg(test)]

mod tests{

    use std::collections::HashMap;

    #[cfg(test)]

    // our functions are getting lines without starting letters, so the examples are like "42e" instead of "i42e"
    #[test]
    fn parsing_positive_int(){
        let mut index = 0;
        assert_eq!(crate::torrent_file_parser::parse_int(&"42e".to_string().as_bytes().to_vec(), &mut index), 42);
        assert_eq!(index, 3);
    }

    #[test]
    fn parsing_zero_int(){
        let mut index = 0;
        assert_eq!(crate::torrent_file_parser::parse_int(&"0e".to_string().as_bytes().to_vec(), &mut index), 0);
        assert_eq!(index, 2);
    }

    #[test]
    fn parsing_negative_int(){
        let mut index = 0;
        assert_eq!(crate::torrent_file_parser::parse_int(&"-75637e".to_string().as_bytes().to_vec(), &mut index), -75637);
        assert_eq!(index, 7);
    }

    #[test]
    fn parsing_string_1(){
        let mut index = 0;
        assert_eq!(crate::torrent_file_parser::parse_string(&"4:spam".to_string().as_bytes().to_vec(), &mut index), "spam");
        assert_eq!(index, 6);
    }

    #[test]
    fn parsing_string_2(){
        let mut index = 0;
        assert_eq!(crate::torrent_file_parser::parse_string(&"13:parrot sketch".to_string().as_bytes().to_vec(), &mut index), "parrot sketch");
        assert_eq!(index, 16);
    }

    #[test]
    fn parsing_list(){
        let mut index = 0;
        let result: Vec<super::Content> = {
            crate::torrent_file_parser::parse_list(&"13:parrot sketchi42ee".to_string().as_bytes().to_vec(), &mut index)
        };
        assert_eq!(result[0], super::Content::Str("parrot sketch".to_string()));
        assert_eq!(result[1], super::Content::Int(42));
        assert_eq!(index, 21);
    }

    #[test]
    fn parsing_dict(){
        let mut index = 0;
        let result: HashMap<String, super::Content> = {
            crate::torrent_file_parser::parse_dict(&"3:bar4:spam3:fooi42ee".to_string().as_bytes().to_vec(), &mut index)
        };
        assert_eq!(*result.get("bar").unwrap(), super::Content::Str("spam".to_string()));
        assert_eq!(*result.get("foo").unwrap(), super::Content::Int(42));
        assert_eq!(index, 21);
    }

    #[test]
    fn testing_info_hash(){
        let mut index = 0;
        let example = "4:info4:spam3:fooi42ee".to_string().as_bytes().to_vec();
        let _ = crate::torrent_file_parser::parse_dict(&example, &mut index);
        unsafe {
            assert_eq!(crate::torrent_file_parser::INFO_START, 6);
            assert_eq!(crate::torrent_file_parser::INFO_END, 12);
        }
        assert_eq!(crate::torrent_file_parser::create_info_hash(&example),
                   vec![151, 39, 109, 243, 254, 149, 209, 1, 232, 44, 41, 51, 88, 33, 38, 89, 2, 164, 15, 144]);
    }

    #[test]
    fn testing_info_hash_2(){
        let mut index = 0;
        let example = "4:infod5:filesld6:lengthi615e4:patheeee".to_string().as_bytes().to_vec();
        let _ = crate::torrent_file_parser::parse_dict(&example, &mut index);
        unsafe {
            assert_eq!(crate::torrent_file_parser::INFO_START, 6);
            assert_eq!(crate::torrent_file_parser::INFO_END, 38);
        }
        assert_eq!(crate::torrent_file_parser::create_info_hash(&example),
                   vec![4, 126, 211, 231, 220, 45, 82, 116, 37, 135, 96, 198, 181, 86, 85, 175, 170, 126, 67, 178]);
    }
}
