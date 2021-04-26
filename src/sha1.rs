pub fn sha1_encode(bytes: &Vec<u8>) -> Vec<u8>{
    let mut h0: u64 = 0x67452301;
    let mut h1: u64 = 0xEFCDAB89;
    let mut h2: u64 = 0x98BADCFE;
    let mut h3: u64 = 0x10325476;
    let mut h4: u64 = 0xC3D2E1F0;

    let original_length_bits = 8*(bytes.len() as u64);
    let mut number_of_bytes = bytes.len();
    let mut copied_bytes: Vec<u8> = Vec::new();

    for byte in bytes.iter(){
        copied_bytes.push(*byte);
    }

    if number_of_bytes%64 < 56 && number_of_bytes%64 != 0{
        number_of_bytes += 64 - number_of_bytes%64;
    }
    else if number_of_bytes%64 >= 56{
        number_of_bytes += 128 - number_of_bytes%64;
    }

    for i in 0..(number_of_bytes - bytes.len()){
        if i == 0{
            copied_bytes.push(1);
        }
        else if i == (number_of_bytes - bytes.len()) - 8{
            for byte in original_length_bits.to_be_bytes().iter(){
                copied_bytes.push(*byte);
            }
            break;
        }
        else{
            copied_bytes.push(0);
        }
    }

    let mut batch: Vec<u32> = Vec::new();
    batch.reserve(80);
    let mut big_endian: [u8; 4] = [0, 0, 0, 0];

    let mut a = h0;
    let mut b = h1;
    let mut c = h2;
    let mut d = h3;
    let mut e = h4;

    let mut f: u64;
    let mut k: u64;

    let mut high_bits_nuller: u64 = 0;
    for i in 0..32{
        high_bits_nuller &= 1 << i;
    }

    for (index, byte) in bytes.iter().enumerate(){
        big_endian[index%4] = *byte;
        if index%4 == 3{
            batch.push(u32::from_be_bytes(big_endian));
        }
        if index%64 == 63{
            for i in 16..80{
                batch[i] = (batch[i-3] ^ batch[i-8] ^ batch[i-14] ^ batch[i-16]) << 1;
            }
            for i in 0..80{
                if i <= 19{
                    f = (b & c) | ((!b) & d);
                    k = 0x5A827999;
                }
                else if i >= 20 && i <= 39{
                    f = b ^ c ^ d;
                    k = 0x6ED9EBA1;
                }
                else if i >= 40 && i<=59{
                    f = (b & c) | (b & d) | (c & d);
                    k = 0x8F1BBCDC;
                }
                else{
                    f = b ^ c ^ d;
                    k = 0xCA62C1D8;
                }
                let tmp = (a << 5) + f + e + k + batch[i] as u64;
                e = d;
                d = c;
                c = b << 30;
                b = a;
                a = tmp & high_bits_nuller;
            }
            h0 = (h0 + a) & high_bits_nuller;
            h1 = (h1 + a) & high_bits_nuller;
            h2 = (h2 + a) & high_bits_nuller;
            h3 = (h3 + a) & high_bits_nuller;
            h4 = (h4 + a) & high_bits_nuller;
            batch = Vec::new();
        }
    }
    let mut hash: Vec<u8> = Vec::new();
    for (index, byte) in h0.to_be_bytes().iter().enumerate(){
        if index >= 4{
            hash.push(*byte);
        }
    }
    for (index, byte) in h1.to_be_bytes().iter().enumerate(){
        if index >= 4{
            hash.push(*byte);
        }
    }
    for (index, byte) in h2.to_be_bytes().iter().enumerate(){
        if index >= 4{
            hash.push(*byte);
        }
    }
    for (index, byte) in h3.to_be_bytes().iter().enumerate(){
        if index >= 4{
            hash.push(*byte);
        }
    }
    for (index, byte) in h4.to_be_bytes().iter().enumerate(){
        if index >= 4{
            hash.push(*byte);
        }
    }
    hash
}

#[cfg(test)]

#[test]
fn test_sha1_1() {
    let example = "The quick brown fox jumps over the lazy dog".to_string().as_bytes().to_vec();
    assert_eq!(sha1_encode(&example), vec![47, 212, 225, 198, 122, 45, 40, 252, 237, 132, 158, 225, 187, 118, 231, 57, 27, 147, 235, 18])
}
