use std::fs;
use std::env;
use serde_json::json;
use serde_json::Value;


static TAG_END: u8 = 0x00;
static TAG_BYTE: u8 = 0x01;
static TAG_SHORT: u8 = 0x02;
static TAG_INT: u8 = 0x03;
static TAG_LONG: u8 = 0x04;
static TAG_FLOAT: u8 = 0x05;
static TAG_DOUBLE: u8 = 0x06;
static TAG_BYTE_ARRAY: u8 = 0x07;
static TAG_STRING: u8 = 0x08;
static TAG_LIST: u8 = 0x09;
static TAG_COMPOUND: u8 = 0x0A;
static TAG_INT_ARRAY: u8 = 0x0B;
static TAG_LONG_ARRAY: u8 = 0x0C;


fn read_tag_byte(bytes_data: &Vec<u8>, tag_start_index: usize) -> (i8, usize) {
    let byte_unsigned: u8 = bytes_data[tag_start_index];
    let _byte: i8 = i8::from_be_bytes([byte_unsigned]);

    return (_byte, 1);
}

fn read_tag_short(bytes_data: &Vec<u8>, tag_start_index: usize) -> (i16, usize) {
    let short_start: usize = tag_start_index;
    let short_bytes: &Vec<u8> = &(bytes_data[short_start ..short_start + 2]).to_vec();
    let short_value: i16 = i16::from_be_bytes([short_bytes[0], short_bytes[1]]);

    return (short_value, 2); // 2 bytes of data per short
}

fn read_tag_int(bytes_data: &Vec<u8>, tag_start_index: usize) -> (i32, usize) {
    let int_start: usize = tag_start_index;
    let int_bytes: &Vec<u8> = &(bytes_data[int_start .. int_start + 4]).to_vec();
    let int_value: i32 = i32::from_be_bytes([int_bytes[0], int_bytes[1], int_bytes[2], int_bytes[3]]);

    return (int_value, 4); // 4 bytes of data per int
}

fn read_tag_long(bytes_data: &Vec<u8>, tag_start_index: usize) -> (i64, usize) {
    let long_start: usize = tag_start_index;
    let long_bytes: &Vec<u8> = &(bytes_data[long_start .. long_start + 8]).to_vec();
    let long_value: i64 = i64::from_be_bytes([
        long_bytes[0], long_bytes[1], long_bytes[2], long_bytes[3],
        long_bytes[4], long_bytes[5], long_bytes[6], long_bytes[7]
    ]);

    return (long_value, 8); // 8 bytes of data per long;
}

fn read_tag_float(bytes_data: &Vec<u8>, tag_start_index: usize) -> (f32, usize) {
    let float_start: usize = tag_start_index;
    let float_bytes: &Vec<u8> = &(bytes_data[float_start .. float_start + 4]).to_vec();
    let float_value: f32 = f32::from_be_bytes([
        float_bytes[0], float_bytes[1], float_bytes[2], float_bytes[3]
    ]);

    return (float_value, 4); // 4 bytes of data per double;
}

fn read_tag_double(bytes_data: &Vec<u8>, tag_start_index: usize) -> (f64, usize) {
    let double_start: usize = tag_start_index;
    let double_bytes: &Vec<u8> = &(bytes_data[double_start .. double_start + 8]).to_vec();
    let double_value: f64 = f64::from_be_bytes([
        double_bytes[0], double_bytes[1], double_bytes[2], double_bytes[3],
        double_bytes[4], double_bytes[5], double_bytes[6], double_bytes[7]
    ]);

    return (double_value, 8); // 8 bytes of data per double;
}

fn read_tag_byte_array(bytes_data: &Vec<u8>, tag_start_index: usize) -> (Vec<Value>, usize) {
    let mut bytes_read: usize = 0;

    let item_count_start: usize = tag_start_index;
    let (item_count_value, bytes_read_for_size): (i32, usize) = read_tag_int(bytes_data, item_count_start);
    bytes_read += bytes_read_for_size;

    let mut payload_values: Vec<Value> = Default::default();
    let mut bytes_offset = item_count_start + bytes_read_for_size;

    for _payload_value_index in 0 .. item_count_value {
        let (item_value, item_size): (Value, usize) = decode_tag(bytes_data, bytes_offset, TAG_BYTE);
        bytes_offset += item_size;
        bytes_read += item_size;

        payload_values.push(item_value);
    }
    return (payload_values, bytes_read);
}

fn read_tag_string(bytes_data: &Vec<u8>, start_index: usize) -> (String, usize) {
    let size_bytes: &[u8] = &bytes_data[start_index .. start_index+2];
    let name_length: u16 = u16::from_be_bytes([size_bytes[0], size_bytes[1]]);

    let name_start: usize = start_index + 2;
    let name_bytes: &[u8] = &bytes_data[name_start .. name_start + usize::from(name_length)];
    let name: String = String::from_utf8(name_bytes.to_vec()).unwrap();

    let bytes_read: usize = usize::from(2 + name_length);
    return (name, bytes_read);
}

fn read_tag_list(bytes_data: &Vec<u8>, tag_start_index: usize) -> (Vec<Value>, usize) {
    let mut bytes_read: usize = 0;

    let payload_start = tag_start_index;
    let list_value_type_id: &u8 = &(bytes_data[payload_start]);
    bytes_read += 1;

    let item_count_start: usize = payload_start + 1;
    let (item_count_value, bytes_read_for_size): (i32, usize) = read_tag_int(bytes_data, item_count_start);
    bytes_read += bytes_read_for_size;

    let mut payload_values: Vec<Value> = Default::default();
    let mut bytes_offset = item_count_start + bytes_read_for_size;

    for _payload_value_index in 0 .. item_count_value {
        let (item_value, item_size): (Value, usize) = decode_tag(bytes_data, bytes_offset, *list_value_type_id);
        bytes_offset += item_size;
        bytes_read += item_size;

        payload_values.push(item_value);
    }
    return (payload_values, bytes_read);
}

fn read_tag_compound(bytes_data: &Vec<u8>, tag_start_index: usize) -> (Value, usize) {
    let mut json_data = json!({});
    let mut current_byte: usize = tag_start_index + 0;

    loop {
        let tag_id_byte: u8 = bytes_data[current_byte];
        current_byte += 1;

        if tag_id_byte == TAG_END {
            break;
        }

        let (tag_name, tag_name_length): (String, usize) = read_tag_string(&bytes_data, current_byte);
        current_byte += tag_name_length;

        let (value, bytes_read): (Value, usize) = decode_tag(&bytes_data, current_byte, tag_id_byte);
        current_byte += bytes_read;

        //println!("Index: {current_byte}, ID Byte: {tag_id_byte}\tTag Name: {tag_name}, Value: {value}, Bytes Read: {bytes_read}");

        json_data[tag_name.to_string()] = value;
    }

    //let json_string = serde_json::to_string_pretty(&json_data).unwrap();
    //println!("{json_string}");

    let bytes_read = current_byte - tag_start_index;
    return (json_data, bytes_read);
}

fn read_tag_int_array(bytes_data: &Vec<u8>, tag_start_index: usize) -> (Vec<Value>, usize) {
    let mut bytes_read: usize = 0;

    let item_count_start: usize = tag_start_index;
    let (item_count_value, bytes_read_for_size): (i32, usize) = read_tag_int(bytes_data, item_count_start);
    bytes_read += bytes_read_for_size;

    let mut payload_values: Vec<Value> = Default::default();
    let mut bytes_offset = item_count_start + bytes_read_for_size;

    for _payload_value_index in 0 .. item_count_value {
        let (item_value, item_size): (Value, usize) = decode_tag(bytes_data, bytes_offset, TAG_INT);
        bytes_offset += item_size;
        bytes_read += item_size;

        payload_values.push(item_value);
    }
    return (payload_values, bytes_read);
}

fn read_tag_long_array(bytes_data: &Vec<u8>, tag_start_index: usize) -> (Vec<Value>, usize) {
    let mut bytes_read: usize = 0;

    let item_count_start: usize = tag_start_index;
    let (item_count_value, bytes_read_for_size): (i32, usize) = read_tag_int(bytes_data, item_count_start);
    bytes_read += bytes_read_for_size;

    let mut payload_values: Vec<Value> = Default::default();
    let mut bytes_offset = item_count_start + bytes_read_for_size;

    for _payload_value_index in 0 .. item_count_value {
        let (item_value, item_size): (Value, usize) = decode_tag(bytes_data, bytes_offset, TAG_LONG);
        bytes_offset += item_size;
        bytes_read += item_size;

        payload_values.push(item_value);
    }
    return (payload_values, bytes_read);
}

fn decode_tag(bytes_data: &Vec<u8>, tag_start_index: usize, tag_type: u8) -> (Value, usize) {
    let mut output_value: Value = Value::from(0);
    let mut bytes_read: usize = 0;

    if tag_type == TAG_BYTE {
        let (output, bytes_read_out) = read_tag_byte(bytes_data, tag_start_index);

        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_SHORT {
        let (output, bytes_read_out) = read_tag_short(bytes_data, tag_start_index);
        
        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_INT {
        let (output, bytes_read_out) = read_tag_int(bytes_data, tag_start_index);
        
        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_LONG{
        let (output, bytes_read_out) = read_tag_long(bytes_data, tag_start_index);
        
        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_FLOAT {
        let (output, bytes_read_out) = read_tag_float(bytes_data, tag_start_index);

        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_DOUBLE {
        let (output, bytes_read_out) = read_tag_double(bytes_data, tag_start_index);

        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_BYTE_ARRAY {
        let (output, bytes_read_out) = read_tag_byte_array(bytes_data, tag_start_index);

        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_STRING {
        let (output, bytes_read_out) = read_tag_string(bytes_data, tag_start_index);

        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_LIST{
        let (output, bytes_read_out) = read_tag_list(bytes_data, tag_start_index);

        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_COMPOUND {
        let (output, bytes_read_out) = read_tag_compound(bytes_data, tag_start_index);

        output_value = output;
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_INT_ARRAY {
        let (output, bytes_read_out) = read_tag_int_array(bytes_data, tag_start_index);

        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else if tag_type == TAG_LONG_ARRAY {
        let (output, bytes_read_out) = read_tag_long_array(bytes_data, tag_start_index);

        output_value = Value::from(output);
        bytes_read = bytes_read_out;
    }
    else{
        //print!("UNKNOWN TAG ID: {tag_type}");
        unimplemented!("UNKNOWN TAG ID: {tag_type}");
    }

    return (output_value, bytes_read);
}

fn decode_nbt_data(bytes_data: &Vec<u8>) -> Value {
    // First 3 bytes are init I think? So I am going to skip them
    // 0a 00 00 | 0a = TAG_Compound, then 00 00 to signify the length of the name.
    //                  Name length is zero in total so just skip it to create the overarching json I guess
    let mut json_data = json!({});
    let mut n: usize = 3;
    //let mut tags_read: usize = 0;
    let length_of_bytes_data: usize = bytes_data.len();

    while n < length_of_bytes_data {
        let mut n_to_add: usize = 1;

        let tag_id_byte: u8 = bytes_data[n];

        if tag_id_byte == TAG_END {
            // this is probably the end of the file!
            break;
        }
        
        let tag_name: &String = &read_tag_string(&bytes_data, n+1).0;
        let name_offset: usize = 2 +  tag_name.len();
        n_to_add += name_offset;

        let decode_start = n + name_offset + 1;
        let (value, bytes_read): (Value, usize) = decode_tag(&bytes_data, decode_start, tag_id_byte);

        n_to_add += bytes_read;

        //println!("{value} | {bytes_read}");

        json_data[tag_name.to_string()] = value;

        //println!("Tag Number: {tags_read}, Index: {n}, Byte: {tag_id_byte}, Tag Name: {tag_name}");

        n += n_to_add;
        //tags_read += 1;
    }

    let json_string = serde_json::to_string(&json_data).unwrap();
    let json_string_pretty = serde_json::to_string_pretty(&json_data).unwrap();
    //println!("{json_string}");

    return json_data;
}


fn main() {
    let args: Vec<String> = env::args().collect();
    //dbg!(args);

    let mut input_file_path: String = "".to_string();
    let mut output_file_path: String = "".to_string();

    for arg in args.iter() {
        let is_input = arg.to_lowercase().starts_with("--input=");
        let is_output = arg.to_lowercase().starts_with("--output=");

        if is_input == false && is_output == false {
            continue;
        }

        let file_path_splits: Vec<&str> = arg.split("=").collect();
        let file_path: &str = file_path_splits[1];
        
        if is_input {
            input_file_path = file_path.to_string();
        }
        else{
            output_file_path = file_path.to_string();
        }
    }

    //dbg!(&input_file_path, &output_file_path);
    assert_ne!(input_file_path, "".to_string());

    let nbt_bytes = fs::read(input_file_path)
        .expect("Should have been able to read the file!");

    let json_data: Value = decode_nbt_data(&nbt_bytes);

    if output_file_path != "".to_string(){
        // only write if we have an output file!
        let json_string_pretty: String = serde_json::to_string_pretty(&json_data).unwrap();

        fs::write(output_file_path, json_string_pretty)
            .expect("Should have been able to write the file!");
    }

    // output incase we want to read it from the console
    let json_string: String = serde_json::to_string(&json_data).unwrap();
    print!("{json_string}");
}

// 